use std::{
    net::{Ipv4Addr, Ipv6Addr},
    ops::BitXor,
};

use axum::extract::Query;
use serde::Deserialize;
use std::iter::zip;

#[derive(Deserialize)]
pub struct DestParams {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

pub async fn dest(Query(params): Query<DestParams>) -> String {
    let octets: [u8; 4] = params
        .from
        .octets()
        .into_iter()
        .zip(params.key.octets().into_iter())
        .map(|(from, key)| from.overflowing_add(key).0)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    Ipv4Addr::from(octets).to_string()
}

#[derive(Deserialize)]
pub struct KeyParams {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

pub async fn key(Query(params): Query<KeyParams>) -> String {
    let octets: [u8; 4] = params
        .from
        .octets()
        .into_iter()
        .zip(params.to.octets().into_iter())
        .map(|(from, to)| to.overflowing_sub(from).0)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    // dbg!(&octets);
    Ipv4Addr::from(octets).to_string()
}

#[derive(Deserialize)]
pub struct DestV6Params {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

pub async fn dest_v6(Query(params): Query<DestV6Params>) -> String {
    let octets: [u8; 16] = zip(params.from.octets(), params.key.octets())
        .map(|(from, key)| from.bitxor(key))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    Ipv6Addr::from(octets).to_string()
}

#[derive(Deserialize)]
pub struct KeyV6Params {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

pub async fn key_v6(Query(params): Query<KeyV6Params>) -> String {
    let octets: [u8; 16] = zip(params.from.octets(), params.to.octets())
        .map(|(from, to)| to.bitxor(from))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    Ipv6Addr::from(octets).to_string()
}
