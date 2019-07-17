use chrono::{Date, DateTime, Datelike, NaiveTime, TimeZone, Utc};

use snafu::{ErrorCompat, ResultExt, Snafu};

pub trait Context {
    type TZ: TimeZone;
    fn tz(&self) -> &Self::TZ;
    fn now(&self) -> &DateTime<Self::TZ>;
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to parse datetime: {}", source))]
    DateTimeParseError { source: chrono::format::ParseError },
}

macro_rules! attempt {
    ($code:expr) => {
        match $code {
            Ok(s) => return Ok(s),
            Err(e) => e,
        }
    };
}

pub fn parse<C: Context>(c: &C, text: &str) -> Result<DateTime<C::TZ>, ()> {
    attempt!(parse_datetime(c.tz(), text));
    match parse_date(c, text) {
        Ok(date) => return Ok(date.and_hms(0, 0, 0)),
        Err(_) => {}
    }
    match parse_time(c, text) {
        Ok(time) => return Ok(c.now().date().and_time(time).unwrap()),
        Err(_) => {}
    }
    Err(())
}

fn parse_datetime<T: TimeZone>(tz: &T, text: &str) -> Result<DateTime<T>, ()> {
    attempt!(tz.datetime_from_str(text, "%Y-%m-%dT%H:%M:%S"));
    Err(())
}

fn parse_date<C: Context>(c: &C, text: &str) -> Result<Date<C::TZ>, ()> {
    match format_parse(fmts::FULL_DATE, text) {
        Ok(parsed) => {
            return Ok(c.tz().ymd(
                parsed.year.unwrap(),
                parsed.month.unwrap(),
                parsed.day.unwrap(),
            ))
        }
        Err(_) => {}
    }
    match format_parse(fmts::PARTIAL_DATE, text) {
        Ok(parsed) => {
            return Ok(c.tz().ymd(
                c.now().with_timezone(c.tz()).year(),
                parsed.month.unwrap(),
                parsed.day.unwrap(),
            ))
        }
        Err(_) => {}
    }
    Err(())
}

fn parse_time<C: Context>(c: &C, text: &str) -> Result<NaiveTime, ()> {
    match format_parse(fmts::HOUR_AND_MINUTE, text) {
        Ok(mut parsed) => {
            parsed.set_second(0);
            return parsed.to_naive_time().map_err(|_| ());
        }
        Err(_) => {}
    }
    Err(())
}

fn format_parse(fmt: &[chrono::format::Item], text: &str) -> Result<chrono::format::Parsed, ()> {
    use chrono::format;
    let fmt_iter = fmt.iter().map(|v| v.clone());
    let mut parsed = format::Parsed::new();
    match format::parse(&mut parsed, text, fmt_iter) {
        Ok(()) => Ok(parsed),
        Err(_) => Err(()),
    }
}

mod fmts {
    use chrono::format::{Item, Numeric::*, Pad};

    pub const FULL_DATE: &[Item] = &[
        Item::Numeric(Year, Pad::None),
        Item::Literal("-"),
        Item::Numeric(Month, Pad::None),
        Item::Literal("-"),
        Item::Numeric(Day, Pad::None),
    ];

    pub const PARTIAL_DATE: &[Item] = &[
        Item::Literal("--"),
        Item::Numeric(Month, Pad::None),
        Item::Literal("-"),
        Item::Numeric(Day, Pad::None),
    ];

    pub const HOUR_AND_MINUTE: &[Item] = &[
        Item::Numeric(Hour, Pad::None),
        Item::Literal(":"),
        Item::Numeric(Minute, Pad::None),
    ];

}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;

    struct DummyContext(DateTime<Utc>);
    impl Context for DummyContext {
        type TZ = Utc;
        fn tz(&self) -> &Self::TZ {
            &Utc
        }
        fn now(&self) -> &DateTime<Self::TZ> {
            &self.0
        }
    }
    impl DummyContext {
        fn new() -> Self {
            DummyContext(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0))
        }
    }

    #[test]
    fn full_datetime() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0)),
            parse(&DummyContext::new(), "2019-07-16T19:25:00")
        );
    }

    #[test]
    fn just_the_date() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(0, 0, 0)),
            parse(&DummyContext::new(), "2019-07-16")
        );
    }

    #[test]
    fn just_the_month_and_day_no_padding() {
        assert_eq!(
            Ok(Utc.ymd(2019, 7, 6).and_hms(0, 0, 0)),
            parse(&DummyContext::new(), "--7-6")
        );
    }

    #[test]
    fn just_the_month_and_day() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(0, 0, 0)),
            parse(&DummyContext::new(), "--07-16")
        );
    }

    #[test]
    fn just_hour_and_minute() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0)),
            parse(&DummyContext::new(), "19:25")
        );
    }
}
