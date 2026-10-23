pub const OID_SYS_DESCR: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 1, 0];
pub const OID_HR_DEVICE_DESCR: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 2, 1, 3, 1];
pub const OID_HR_DEVICE_STATUS: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 2, 1, 5, 1];
pub const OID_HR_PRINTER_STATUS: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 1, 1];
pub const OID_HR_PRINTER_ERROR_STATE: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 2, 1];
pub const OID_PRT_GENERAL_PRINTER_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 16, 1];
pub const OID_PRT_GENERAL_SERIAL_NUMBER: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 17, 1];
pub const OID_PRT_INPUT_VENDOR_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 8, 2, 1, 14, 1, 1];
pub const OID_PRT_MARKER_LIFE_COUNT: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 4, 1, 1];
pub const OID_PRT_MARKER_PROCESS_COLORANTS: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 6, 1, 1];
pub const OID_PRT_CONSOLE_LINE_1: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 16, 5, 1, 2, 1, 1];
pub const OID_PRT_CONSOLE_LINE_2: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 16, 5, 1, 2, 1, 2];
pub const OID_ENT_PHYSICAL_MFG_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 1, 12, 1];
pub const OID_BROTHER_MAINTENANCE: &[u32] =
    &[1, 3, 6, 1, 4, 1, 2435, 2, 3, 9, 4, 2, 1, 5, 5, 8, 0];

pub const OID_PRT_MARKER_SUPPLIES_ENTRY: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 11, 1, 1];
pub const SUPPLY_COL_COLORANT_INDEX: u32 = 3;
pub const SUPPLY_COL_TYPE: u32 = 5;
pub const SUPPLY_COL_DESCRIPTION: u32 = 6;
pub const SUPPLY_COL_MAX_CAPACITY: u32 = 8;
pub const SUPPLY_COL_LEVEL: u32 = 9;
pub const SUPPLY_WALK_COLUMNS: &[u32] = &[
    SUPPLY_COL_COLORANT_INDEX,
    SUPPLY_COL_TYPE,
    SUPPLY_COL_DESCRIPTION,
    SUPPLY_COL_MAX_CAPACITY,
    SUPPLY_COL_LEVEL,
];

pub const OID_PRT_MARKER_COLORANT_VALUE: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 12, 1, 1, 4];

pub const OID_PRT_MEDIA_PATH_ENTRY: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 13, 4, 1];
pub const MEDIA_PATH_COL_TYPE: u32 = 9;
pub const MEDIA_PATH_COL_DESCRIPTION: u32 = 10;
pub const MEDIA_PATH_WALK_COLUMNS: &[u32] = &[MEDIA_PATH_COL_TYPE, MEDIA_PATH_COL_DESCRIPTION];

pub const OID_PRT_ALERT_ENTRY: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 18, 1, 1];
pub const ALERT_COL_SEVERITY: u32 = 2;
pub const ALERT_COL_GROUP: u32 = 5;
pub const ALERT_COL_CODE: u32 = 7;
pub const ALERT_COL_DESCRIPTION: u32 = 8;
pub const ALERT_WALK_COLUMNS: &[u32] = &[
    ALERT_COL_SEVERITY,
    ALERT_COL_GROUP,
    ALERT_COL_CODE,
    ALERT_COL_DESCRIPTION,
];

pub const ALERT_CODE_JAM: i64 = 8;
pub const ALERT_CODE_SUBUNIT_EMPTY: i64 = 13;
pub const ALERT_CODE_INPUT_MEDIA_EMPTY: i64 = 808;
pub const ALERT_CODE_INPUT_MANUAL_REQUEST: i64 = 810;
pub const ALERT_GROUP_INPUT: i64 = 8;
pub const ALERT_SEVERITY_WARNING: i64 = 4;
