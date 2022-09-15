// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use chrono::{DateTime, NaiveDateTime, Utc};

pub fn format_gh_to_th(amount: f64) -> String {
    let mut number: String = format_number((amount / 1000.0) as usize);
    number.push_str(" Th/s");
    number
}

pub fn format_btc_to_sats(amount: f64) -> String {
    format_sats((amount * 100_000_000.0) as u64)
}

pub fn format_sats(amount: u64) -> String {
    let mut number: String = format_number(amount as usize);
    number.push_str(" SAT");
    number
}

pub fn format_number(num: usize) -> String {
    let mut number: String = num.to_string();
    let number_len: usize = number.len();

    if number_len > 3 {
        let mut counter: u8 = 1;
        loop {
            if num / usize::pow(1000, counter.into()) > 0 {
                counter += 1;
            } else {
                break;
            }
        }

        counter -= 1;

        let mut formatted_number: Vec<String> =
            vec![number[0..(number_len - counter as usize * 3)].into()];

        number.replace_range(0..(number_len - counter as usize * 3), "");

        loop {
            if counter > 0 {
                if !number[0..3].is_empty() {
                    formatted_number.push(number[0..3].into());
                    number.replace_range(0..3, "");
                }

                counter -= 1
            } else {
                break;
            }
        }

        number = formatted_number.join(",");
    }

    number
}

pub fn timestamp_to_utc_datetime(timestamp: i64) -> DateTime<Utc> {
    let nt = NaiveDateTime::from_timestamp(timestamp, 0);
    DateTime::from_utc(nt, Utc)
}

pub fn format_date(timestamp: i64, fmt: &str) -> String {
    let dt = timestamp_to_utc_datetime(timestamp);
    dt.format(fmt).to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_date() {
        assert_eq!(
            format_date(1646649012, "%Y-%m-%d"),
            "2022-03-07".to_string()
        );
    }

    #[test]
    fn format_num() {
        assert_eq!(format_number(180000), "180,000".to_string());
    }

    #[test]
    fn test_format_gh_to_th() {
        assert_eq!(format_gh_to_th(1000.0), "1 Th/s".to_string());
        assert_eq!(format_gh_to_th(1000000.0), "1,000 Th/s".to_string());
        assert_eq!(
            format_gh_to_th(5820970883.3011),
            "5,820,970 Th/s".to_string()
        );
    }

    #[test]
    fn format_satoshi() {
        assert_eq!(format_sats(100), "100 SAT".to_string());
        assert_eq!(format_sats(1000), "1,000 SAT".to_string());
        assert_eq!(format_sats(10000), "10,000 SAT".to_string());
        assert_eq!(format_sats(100000), "100,000 SAT".to_string());
        assert_eq!(format_sats(1000000), "1,000,000 SAT".to_string());
        assert_eq!(format_sats(1000000000), "1,000,000,000 SAT".to_string());
    }

    #[test]
    fn format_btc_to_satoshi() {
        assert_eq!(format_btc_to_sats(0.00000001), "1 SAT".to_string());
        assert_eq!(format_btc_to_sats(0.00001), "1,000 SAT".to_string());
        assert_eq!(format_btc_to_sats(0.0001), "10,000 SAT".to_string());
        assert_eq!(format_btc_to_sats(0.001), "100,000 SAT".to_string());
        assert_eq!(format_btc_to_sats(0.01), "1,000,000 SAT".to_string());
        assert_eq!(format_btc_to_sats(1.0), "100,000,000 SAT".to_string());
        assert_eq!(format_btc_to_sats(10.0), "1,000,000,000 SAT".to_string());
    }
}
