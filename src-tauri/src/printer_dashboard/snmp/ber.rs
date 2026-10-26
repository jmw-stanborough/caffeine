use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

pub const SNMP_TIMEOUT: Duration = Duration::from_millis(750);
pub const PDU_GET_REQUEST: u8 = 0xA0;
pub const PDU_GET_NEXT_REQUEST: u8 = 0xA1;
pub const PDU_GET_RESPONSE: u8 = 0xA2;

const ASN1_INTEGER: u8 = 0x02;
const ASN1_OCTET_STRING: u8 = 0x04;
const ASN1_NULL: u8 = 0x05;
const ASN1_OID: u8 = 0x06;
const ASN1_SEQUENCE: u8 = 0x30;
const ASN1_COUNTER32: u8 = 0x41;
const ASN1_GAUGE32: u8 = 0x42;
const ASN1_TIMETICKS: u8 = 0x43;

pub const SNMP_VERSION_1: i32 = 0;
pub const SNMP_VERSION_2C: i32 = 1;
const SNMP_UDP_BUFFER_SIZE: usize = 4096;

const BER_LONG_LENGTH_FLAG: u8 = 0x80;
const BER_LENGTH_VALUE_MASK: u8 = 0x7F;
const BER_LENGTH_ONE_OCTET: u8 = 0x81;
const BER_LENGTH_TWO_OCTETS: u8 = 0x82;
const MAX_BER_LENGTH_BYTES: usize = 4;

const OID_FIRST_OCTET_RADIX: u32 = 40;
const DEFAULT_ROOT_OID_OCTET: u8 = 43;
const OID_BASE128_CONTINUATION_BIT: u8 = 0x80;
const OID_BASE128_VALUE_MASK: u32 = 0x7F;
const OID_BASE128_SHIFT: u32 = 7;
const SIGN_BIT_MASK: u8 = 0x80;
const BITS_PER_BYTE: usize = 8;

const PRINTABLE_ASCII_MIN: u8 = 0x20;

#[derive(Debug, Clone)]
pub enum SnmpVal {
    Int(i64),
    Bytes(Vec<u8>),
}

impl SnmpVal {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SnmpVal::Int(v) => Some(*v),
            SnmpVal::Bytes(b) => std::str::from_utf8(b).ok()?.trim().parse::<i64>().ok(),
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self {
            SnmpVal::Bytes(b) => Some(clean_octet_string(b)),
            SnmpVal::Int(_) => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            SnmpVal::Bytes(b) => Some(b.as_slice()),
            SnmpVal::Int(_) => None,
        }
    }
}

pub fn encode_length(len: usize) -> Vec<u8> {
    if len < usize::from(BER_LONG_LENGTH_FLAG) {
        vec![len as u8]
    } else if len <= usize::from(u8::MAX) {
        vec![BER_LENGTH_ONE_OCTET, len as u8]
    } else {
        vec![
            BER_LENGTH_TWO_OCTETS,
            (len >> BITS_PER_BYTE) as u8,
            (len & usize::from(u8::MAX)) as u8,
        ]
    }
}

pub fn encode_tlv(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend(encode_length(payload.len()));
    out.extend_from_slice(payload);
    out
}

pub fn encode_integer(val: i32) -> Vec<u8> {
    let bytes = val.to_be_bytes();
    let mut start = 0;
    // Strip redundant sign-extension bytes while preserving two's-complement sign bit.
    while start < bytes.len() - 1 {
        let sign_bit_matches = (bytes[start] == 0x00 && (bytes[start + 1] & SIGN_BIT_MASK) == 0)
            || (bytes[start] == 0xFF && (bytes[start + 1] & SIGN_BIT_MASK) != 0);
        if sign_bit_matches {
            start += 1;
        } else {
            break;
        }
    }
    encode_tlv(ASN1_INTEGER, &bytes[start..])
}

pub fn encode_oid(oid: &[u32]) -> Vec<u8> {
    if oid.len() < 2 {
        return encode_tlv(ASN1_OID, &[DEFAULT_ROOT_OID_OCTET]);
    }
    let mut body = vec![(oid[0] * OID_FIRST_OCTET_RADIX + oid[1]) as u8];
    for &sub in &oid[2..] {
        if sub == 0 {
            body.push(0);
        } else {
            let mut temp = Vec::new();
            let mut v = sub;
            while v > 0 {
                temp.push((v & OID_BASE128_VALUE_MASK) as u8);
                v >>= OID_BASE128_SHIFT;
            }
            temp.reverse();
            let high_octets = temp.len() - 1;
            for byte in temp.iter_mut().take(high_octets) {
                *byte |= OID_BASE128_CONTINUATION_BIT;
            }
            body.extend(temp);
        }
    }
    encode_tlv(ASN1_OID, &body)
}

pub fn build_snmp_packet(
    version: i32,
    community: &str,
    request_id: i32,
    pdu_tag: u8,
    oids: &[Vec<u32>],
) -> Vec<u8> {
    let mut varbinds_inner = Vec::new();
    for oid in oids {
        let mut vb = encode_oid(oid);
        vb.extend_from_slice(&[ASN1_NULL, 0x00]);
        varbinds_inner.extend(encode_tlv(ASN1_SEQUENCE, &vb));
    }
    let varbind_list = encode_tlv(ASN1_SEQUENCE, &varbinds_inner);

    let mut pdu_inner = Vec::new();
    pdu_inner.extend(encode_integer(request_id));
    pdu_inner.extend(encode_integer(0));
    pdu_inner.extend(encode_integer(0));
    pdu_inner.extend(varbind_list);

    let pdu = encode_tlv(pdu_tag, &pdu_inner);

    let mut msg_inner = Vec::new();
    msg_inner.extend(encode_integer(version));
    msg_inner.extend(encode_tlv(ASN1_OCTET_STRING, community.as_bytes()));
    msg_inner.extend(pdu);

    encode_tlv(ASN1_SEQUENCE, &msg_inner)
}

pub fn parse_ber_tlv(data: &[u8], pos: &mut usize) -> Option<(u8, usize, usize)> {
    if *pos >= data.len() {
        return None;
    }
    let tag = data[*pos];
    *pos += 1;
    if *pos >= data.len() {
        return None;
    }
    let first_len = data[*pos];
    *pos += 1;
    let len = if (first_len & BER_LONG_LENGTH_FLAG) == 0 {
        first_len as usize
    } else {
        let num_bytes = (first_len & BER_LENGTH_VALUE_MASK) as usize;
        if num_bytes == 0 || num_bytes > MAX_BER_LENGTH_BYTES || *pos + num_bytes > data.len() {
            return None;
        }
        let mut l = 0usize;
        for _ in 0..num_bytes {
            l = (l << BITS_PER_BYTE) | (data[*pos] as usize);
            *pos += 1;
        }
        l
    };
    if *pos + len > data.len() {
        return None;
    }
    let val_start = *pos;
    *pos += len;
    Some((tag, val_start, len))
}

pub fn decode_oid(bytes: &[u8]) -> Vec<u32> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let first = u32::from(bytes[0]);
    let mut oid = vec![
        first / OID_FIRST_OCTET_RADIX,
        first % OID_FIRST_OCTET_RADIX,
    ];
    let mut acc = 0u32;
    for &b in &bytes[1..] {
        acc = (acc << OID_BASE128_SHIFT) | (u32::from(b) & OID_BASE128_VALUE_MASK);
        if (b & OID_BASE128_CONTINUATION_BIT) == 0 {
            oid.push(acc);
            acc = 0;
        }
    }
    oid
}

pub fn decode_signed_int(bytes: &[u8]) -> i64 {
    if bytes.is_empty() {
        return 0;
    }
    let mut val = if (bytes[0] & SIGN_BIT_MASK) != 0 {
        -1i64
    } else {
        0i64
    };
    for &b in bytes {
        val = (val << BITS_PER_BYTE) | i64::from(b);
    }
    val
}

pub fn decode_unsigned_int(bytes: &[u8]) -> i64 {
    let mut val = 0i64;
    for &b in bytes {
        val = (val << BITS_PER_BYTE) | i64::from(b);
    }
    val
}

pub fn clean_octet_string(bytes: &[u8]) -> String {
    let trimmed: Vec<u8> = bytes
        .iter()
        .copied()
        .take_while(|&b| b != 0)
        .filter(|&b| b >= PRINTABLE_ASCII_MIN || b == b'\t')
        .collect();
    String::from_utf8_lossy(&trimmed).trim().to_string()
}

#[derive(Debug, Clone)]
pub struct SnmpPduResponse {
    pub error_status: i64,
    pub error_index: i64,
    pub varbinds: Vec<(Vec<u32>, Option<SnmpVal>)>,
}

pub fn parse_snmp_pdu(packet: &[u8]) -> Option<SnmpPduResponse> {
    let mut pos = 0;
    let (seq_tag, seq_start, seq_len) = parse_ber_tlv(packet, &mut pos)?;
    if seq_tag != ASN1_SEQUENCE {
        return None;
    }

    let mut inner = seq_start;
    let seq_end = seq_start + seq_len;

    let _ = parse_ber_tlv(packet, &mut inner)?;
    let _ = parse_ber_tlv(packet, &mut inner)?;
    let (pdu_tag, pdu_start, pdu_len) = parse_ber_tlv(packet, &mut inner)?;
    if pdu_tag != PDU_GET_RESPONSE || inner > seq_end {
        return None;
    }

    let mut pdu_pos = pdu_start;
    let _ = parse_ber_tlv(packet, &mut pdu_pos)?;
    let (_, err_start, err_len) = parse_ber_tlv(packet, &mut pdu_pos)?;
    let error_status = decode_signed_int(&packet[err_start..err_start + err_len]);
    let (_, err_idx_start, err_idx_len) = parse_ber_tlv(packet, &mut pdu_pos)?;
    let error_index = decode_signed_int(&packet[err_idx_start..err_idx_start + err_idx_len]);
    let _ = parse_ber_tlv(packet, &mut pdu_pos)?;
    let (vbl_tag, vbl_start, vbl_len) = parse_ber_tlv(packet, &mut pdu_pos)?;
    if vbl_tag != ASN1_SEQUENCE || pdu_pos > pdu_start + pdu_len {
        return None;
    }

    let mut vbl_pos = vbl_start;
    let vbl_end = vbl_start + vbl_len;
    let mut varbinds = Vec::new();

    while vbl_pos < vbl_end {
        let (vb_tag, vb_start, vb_len) = parse_ber_tlv(packet, &mut vbl_pos)?;
        if vb_tag != ASN1_SEQUENCE {
            break;
        }
        let mut vb_pos = vb_start;
        let (oid_tag, oid_start, oid_len) = parse_ber_tlv(packet, &mut vb_pos)?;
        if oid_tag != ASN1_OID {
            continue;
        }
        let oid = decode_oid(&packet[oid_start..oid_start + oid_len]);

        let (val_tag, val_start, val_len) = parse_ber_tlv(packet, &mut vb_pos)?;
        if vb_pos > vb_start + vb_len {
            continue;
        }
        let val_slice = &packet[val_start..val_start + val_len];

        let val = match val_tag {
            ASN1_INTEGER => Some(SnmpVal::Int(decode_signed_int(val_slice))),
            ASN1_COUNTER32 | ASN1_GAUGE32 | ASN1_TIMETICKS => {
                Some(SnmpVal::Int(decode_unsigned_int(val_slice)))
            }
            ASN1_OCTET_STRING => Some(SnmpVal::Bytes(val_slice.to_vec())),
            _ => None,
        };

        varbinds.push((oid, val));
    }

    Some(SnmpPduResponse {
        error_status,
        error_index,
        varbinds,
    })
}

pub fn parse_snmp_response(packet: &[u8]) -> Option<Vec<(Vec<u32>, Option<SnmpVal>)>> {
    let pdu = parse_snmp_pdu(packet)?;
    if pdu.error_status != 0 {
        return None;
    }
    Some(pdu.varbinds)
}

pub async fn snmp_request_raw(
    sock: &UdpSocket,
    version: i32,
    community: &str,
    req_id: i32,
    pdu_tag: u8,
    oids: &[Vec<u32>],
) -> Option<SnmpPduResponse> {
    let pkt = build_snmp_packet(version, community, req_id, pdu_tag, oids);
    sock.send(&pkt).await.ok()?;
    let mut buf = [0u8; SNMP_UDP_BUFFER_SIZE];
    let n = timeout(SNMP_TIMEOUT, sock.recv(&mut buf))
        .await
        .ok()?
        .ok()?;
    parse_snmp_pdu(&buf[..n])
}

pub async fn snmp_request(
    sock: &UdpSocket,
    version: i32,
    community: &str,
    req_id: i32,
    pdu_tag: u8,
    oids: &[Vec<u32>],
) -> Option<Vec<(Vec<u32>, Option<SnmpVal>)>> {
    let pdu = snmp_request_raw(sock, version, community, req_id, pdu_tag, oids).await?;
    if pdu.error_status != 0 {
        return None;
    }
    Some(pdu.varbinds)
}

pub async fn snmp_walk_columns<F>(
    sock: &UdpSocket,
    version: i32,
    community: &str,
    req_id: &mut i32,
    entry_prefix: &[u32],
    columns: &[u32],
    max_steps: usize,
    mut on_cell: F,
) where
    F: FnMut(Vec<u32>, u32, SnmpVal),
{
    let prefix_len = entry_prefix.len();
    let mut current_oids: Vec<Vec<u32>> = columns
        .iter()
        .map(|&col| {
            let mut oid = entry_prefix.to_vec();
            oid.push(col);
            oid
        })
        .collect();

    for _ in 0..max_steps {
        let Some(step_resp) =
            snmp_request(sock, version, community, *req_id, PDU_GET_NEXT_REQUEST, &current_oids).await
        else {
            break;
        };
        *req_id += 1;

        let mut any_in_column = false;
        let mut next_oids = Vec::with_capacity(current_oids.len());

        for (col_idx, (ret_oid, ret_val)) in step_resp.into_iter().enumerate() {
            let Some(&expected_col) = columns.get(col_idx) else {
                break;
            };

            if ret_oid.len() > prefix_len + 1
                && ret_oid[..prefix_len] == *entry_prefix
                && ret_oid[prefix_len] == expected_col
                && let Some(val) = ret_val
            {
                any_in_column = true;
                let instance_key = ret_oid[prefix_len + 1..].to_vec();
                on_cell(instance_key, expected_col, val);
            }
            next_oids.push(ret_oid);
        }

        if !any_in_column || next_oids.len() != current_oids.len() {
            break;
        }
        current_oids = next_oids;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snmp_ber_encoding_and_response_parsing() {
        let oid = vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 9, 1, 1];
        let encoded_oid = encode_oid(&oid);
        let mut pos = 0;
        let (tag, start, len) = parse_ber_tlv(&encoded_oid, &mut pos).unwrap();
        assert_eq!(tag, ASN1_OID);
        let decoded = decode_oid(&encoded_oid[start..start + len]);
        assert_eq!(decoded, oid);

        let vb_oid = encode_oid(&oid);
        let vb_val = encode_integer(85);
        let mut vb_inner = Vec::new();
        vb_inner.extend(vb_oid);
        vb_inner.extend(vb_val);
        let vb = encode_tlv(ASN1_SEQUENCE, &vb_inner);
        let vbl = encode_tlv(ASN1_SEQUENCE, &vb);

        let mut pdu_inner = Vec::new();
        pdu_inner.extend(encode_integer(101));
        pdu_inner.extend(encode_integer(0));
        pdu_inner.extend(encode_integer(0));
        pdu_inner.extend(vbl);
        let pdu = encode_tlv(PDU_GET_RESPONSE, &pdu_inner);

        let mut msg_inner = Vec::new();
        msg_inner.extend(encode_integer(SNMP_VERSION_2C));
        msg_inner.extend(encode_tlv(ASN1_OCTET_STRING, b"public"));
        msg_inner.extend(pdu);
        let packet = encode_tlv(ASN1_SEQUENCE, &msg_inner);

        let parsed = parse_snmp_response(&packet).expect("valid snmp response");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, oid);
        assert_eq!(parsed[0].1.as_ref().unwrap().as_i64(), Some(85));
    }

    #[test]
    fn test_snmp_v1_packet_and_error_pdu() {
        let oid = vec![1, 3, 6, 1, 2, 1, 1, 1, 0];
        let pkt = build_snmp_packet(SNMP_VERSION_1, "public", 100, PDU_GET_REQUEST, &[oid.clone()]);
        // Sequence tag (0x30) + length, then Integer version (0x02, 0x01, 0x00)
        assert_eq!(pkt[0], ASN1_SEQUENCE);
        let mut pos = 0;
        let (_, _, _) = parse_ber_tlv(&pkt, &mut pos).unwrap();
        let (ver_tag, ver_start, ver_len) = parse_ber_tlv(&pkt, &mut pos).unwrap();
        assert_eq!(ver_tag, ASN1_INTEGER);
        assert_eq!(decode_signed_int(&pkt[ver_start..ver_start + ver_len]), 0);

        let vb_oid = encode_oid(&oid);
        let mut vb_inner = Vec::new();
        vb_inner.extend(vb_oid);
        vb_inner.extend_from_slice(&[ASN1_NULL, 0x00]);
        let vb = encode_tlv(ASN1_SEQUENCE, &vb_inner);
        let vbl = encode_tlv(ASN1_SEQUENCE, &vb);

        let mut pdu_inner = Vec::new();
        pdu_inner.extend(encode_integer(100));
        pdu_inner.extend(encode_integer(2)); // error-status: noSuchName
        pdu_inner.extend(encode_integer(1)); // error-index: 1
        pdu_inner.extend(vbl);
        let pdu = encode_tlv(PDU_GET_RESPONSE, &pdu_inner);

        let mut msg_inner = Vec::new();
        msg_inner.extend(encode_integer(SNMP_VERSION_1));
        msg_inner.extend(encode_tlv(ASN1_OCTET_STRING, b"public"));
        msg_inner.extend(pdu);
        let err_packet = encode_tlv(ASN1_SEQUENCE, &msg_inner);

        assert!(parse_snmp_response(&err_packet).is_none());
        let pdu_resp = parse_snmp_pdu(&err_packet).expect("parsed error pdu");
        assert_eq!(pdu_resp.error_status, 2);
        assert_eq!(pdu_resp.error_index, 1);
    }
}
