//! Tests for direction segmentation.

use super::*;

#[test]
fn splits_origin_and_destination() {
    let (o, d) = split_directions("из германии в алматы");
    assert_eq!(o, "из германии");
    assert_eq!(d, "в алматы");
}

#[test]
fn destination_only() {
    let (o, d) = split_directions("нужно доставить в алматы");
    assert_eq!(o, "");
    assert_eq!(d, "в алматы");
}

#[test]
fn origin_only() {
    let (o, d) = split_directions("груз из германии");
    assert_eq!(o, "из германии");
    assert_eq!(d, "");
}

#[test]
fn no_direction_yields_nothing() {
    let (o, d) = split_directions("12 тонн оборудования");
    assert_eq!(o, "");
    assert_eq!(d, "");
}

#[test]
fn marks_the_customer_changing_destination() {
    let (_, d) = split_directions("нет, в астану");
    assert!(d.contains("астан"));
}
