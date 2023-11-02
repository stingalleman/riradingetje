use chrono::TimeZone;

#[derive(Clone)]
pub struct InfluxConfig {
    pub url: String,
    pub bucket: String,
    pub org: String,
    pub token: String,
}

pub fn convert_tst(tst: dsmr5::types::TST) -> Option<i64> {
    let year: i32 = tst.year.into();

    chrono::Local
        .with_ymd_and_hms(
            year + 2000,
            tst.month.into(),
            tst.day.into(),
            tst.hour.into(),
            tst.minute.into(),
            tst.second.into(),
        )
        .unwrap()
        .timestamp_nanos_opt()
}
