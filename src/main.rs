#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
use rocket_contrib::databases::redis::{self, Commands};
use rocket_contrib::json::Json;
use std::collections::HashMap;

const RANK_DATA_KEY_NAME: &str = "PoemStarsRank";
const MATCH_DATA_KEY_NAME: &str = "PoemStarsMatchKill";
const RANK_DATA_EN_KEY_NAME: &str = "PoemStarsEnRank";
const MATCH_DATA_EN_KEY_NAME: &str = "PoemStarsEnMatchKill";

#[database("poemstarsredis")]
struct PoemStarsRedisDbConn(redis::Connection);

#[derive(Deserialize)]
struct RankPlayEndReqInfo {
    id: String,
    max_score: u32,
}

#[derive(Deserialize)]
struct RankListReqInfo {
    id: String,
    list_type: u32, // 1 rank play, 2 match play
}

#[derive(Serialize)]
struct RankPlayEndResInfo {
    my_rank: u32,
}

#[derive(Serialize)]
struct RankListResInfo {
    my_rank: u32,
    score_value: u32, // 有可能是最高分，有可能是击杀数量
    list_data: HashMap<String, u32>,
}

#[post("/rankplayend/<lang>", format = "application/json", data = "<info>")]
fn handle_rankplay_end(
    lang: String,
    conn: PoemStarsRedisDbConn,
    info: Json<RankPlayEndReqInfo>,
) -> Json<RankPlayEndResInfo> {
    let key: &str = if lang == "zh" {
        RANK_DATA_KEY_NAME
    } else {
        RANK_DATA_EN_KEY_NAME
    };

    let id: &str = &info.id;
    let max_score: u32 = info.max_score;

    let mut my_rank: u32 = 0;

    if let Ok(_result) = conn.zadd::<&str, u32, &str, usize>(key, id, max_score) {
        if let Ok(_result) = conn.zrevrank::<&str, &str, u32>(key, id) {
            my_rank = _result + 1;
        }
    }

    Json(RankPlayEndResInfo { my_rank })
}

#[post("/ranklist/<lang>", data = "<info>")]
fn handle_ranklist(
    lang: String,
    conn: PoemStarsRedisDbConn,
    info: Json<RankListReqInfo>,
) -> Json<RankListResInfo> {
    let key: &str = if info.list_type == 1 {
        if lang == "zh" {
            RANK_DATA_KEY_NAME
        } else {
            RANK_DATA_EN_KEY_NAME
        }
    } else {
        if lang == "zh" {
            MATCH_DATA_KEY_NAME
        } else {
            MATCH_DATA_EN_KEY_NAME
        }
    };

    let mut my_rank: u32 = 0;
    let mut score_value: u32 = 0;
    if let Ok(_result) = conn.zrevrank::<&str, &str, u32>(key, &info.id) {
        my_rank = _result + 1;
    }

    if let Ok(_result) = conn.zscore::<&str, &str, u32>(key, &info.id) {
        score_value = _result;
    }

    let mut list_data: HashMap<String, u32> = HashMap::new();

    if let Ok(_result) = conn.zrevrange_withscores::<&str, HashMap<String, u32>>(key, 0, 99) {
        list_data = _result;
    }

    Json(RankListResInfo {
        my_rank,
        score_value,
        list_data,
    })
}

fn main() {
    rocket::ignite()
        .attach(PoemStarsRedisDbConn::fairing())
        .mount("/", routes![handle_rankplay_end, handle_ranklist])
        .launch();
}
