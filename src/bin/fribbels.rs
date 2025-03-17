use std::collections::HashMap;

use std::fs::File;
use itertools::Itertools;

use serde_json::{Map, Value};

use est_tbp::{Relic, RelicSlot, RelicStat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args().nth(1).expect("path as first arg");
    let input = std::path::Path::new(&input);
    assert!(input.exists());

    let save: Value = serde_json::from_reader(File::open(input)?)?;

    let mut missing_weights = false;

    /*
    Object.values(DB.getMetadata().characters).reduce((acc, cur) => {
      acc[cur.id] = cur.scoringMetadata.stats
      return acc
    }, {})
    */
    let default_weights_str = include_str!("../defaultStatWeights.json");
    let default_weights: Value = serde_json::from_str(default_weights_str)?;

    let relics = {
        let mut relics = HashMap::new();
        for relic in save["relics"].as_array().unwrap() {
            let relic = relic.as_object().unwrap();
            relics.insert(relic["id"].as_str().unwrap().to_string(), relic);
        }
        relics
    };

    for char in save["characters"].as_array().unwrap() {
        let char_id = char["id"].as_str().unwrap().parse().unwrap();
        let char_name_opt = parse_char_id(char_id);
        if char_name_opt.is_none() {
            println!("{} ---------------", char_id);
        } else {
            println!("{} ---------------", char_name_opt.unwrap());
        }

        let mut weights_opt = parse_weights_from_save(&save, char_id);
        if weights_opt.is_none() {weights_opt = parse_default_weights(&default_weights, char_id)}
        if weights_opt.is_none() {weights_opt = parse_optimizer_weights(&save, char_id)}
        if weights_opt.is_none() {
          if !missing_weights {
            println!("No weights available for {}.", char_name_opt.unwrap_or(&format!("character with id {char_id}")));
            println!("A temporary fix is to set the desired weights for the character in the optimiser tab and then launch an optimiser run.");
            println!("For a long term fix either update the source code yourself (fribbels.rs) or contact the developer");
            missing_weights = true;
          }
          continue;
        }
        let weights = weights_opt.unwrap();

        let equipped = char["equipped"].as_object().unwrap().values();

        if equipped.len() == 0 {
          println!("");
          continue;
        }

        println!("weights: {weights:?}");

        equipped
            .filter_map(|id| relics.get(id.as_str().unwrap()))
            .map(|relic| parse_relic(relic))
            .for_each(|relic| {
                let score = relic_score(&relic, &weights);

                let p_main = relic.p_main();
                let p_sub = relic.filtered_p_sub(|r: &_| relic_score(r, &weights) > score);
                let p = p_main * p_sub;

                let est_relic_count = 1.0 / p;
                let tbp_per_relic = 40.0 / 2.1;
                let est_tbp = est_relic_count * tbp_per_relic;

                println!("     est. {:>6.1} days | {:>5.1} score | [{:>10?} {:?}] {}", est_tbp / 240.0, score, relic.slot, relic.main, format_subs(&relic));
            });
        println!();
    }

    println!("press enter to close");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}

fn format_subs(r: &Relic) -> String {
    r.subs.iter()
        .counts()
        .iter()
        .map(|(k, v)| format!("{v}x {k:?}"))
        .join(", ")
}

fn parse_relic(relic: &Map<String, Value>) -> Relic {
    Relic {
        rarity: relic["grade"].as_i64().unwrap() as usize,
        slot: parse_slot(relic["part"].as_str().unwrap()),
        main: parse_stat(relic["main"]["stat"].as_str().unwrap()).unwrap(),
        subs: relic["substats"].as_array().unwrap()
            .iter()
            .flat_map(|sub| {
                let stat = parse_stat(sub["stat"].as_str().unwrap()).unwrap();
                let num = sub["addedRolls"].as_i64().unwrap() as usize + 1;
                std::iter::repeat(stat).take(num)
            })
            .collect(),
    }
}

fn parse_optimizer_weights(save: &Value, id: u32) -> Option<HashMap<RelicStat, f64>> {
  let characters = save["characters"].as_array()?;
  let character = characters.into_iter().find(|&x| {
    let character = x.as_object();
    if character.is_none() {return false;}
    id == character.unwrap()["id"]
  })?;
  let pairs = character["form"]["weights"].as_object()?;
  Some(parse_pairs(pairs))
}

fn parse_weights_from_save(save: &Value, id: u32) -> Option<HashMap<RelicStat, f64>> {
    let pairs = save["scoringMetadataOverrides"][format!("{id}")]["stats"].as_object()?;
    Some(parse_pairs(pairs))
}

fn parse_default_weights(defaults: &Value, id: u32) -> Option<HashMap<RelicStat, f64>> {
  let pairs = defaults[format!("{id}")].as_object()?;
  Some(parse_pairs(pairs))
}

fn parse_pairs(pairs: &Map<String, Value>) -> HashMap<RelicStat, f64> {
    let mut weights = HashMap::new();
    for (k, v) in pairs {
        let stat = parse_stat(k);
        if let Some(stat) = stat {
            let mut w = v.as_f64().unwrap();

            if matches!(stat, RelicStat::Atk | RelicStat::Def | RelicStat::Hp) {
                w *= 0.4;
            }

            if matches!(stat, RelicStat::Spd) {
              w *= 2.3 * 2.59 / (6.48*0.9); // hack to get more accurate speed scores
            }

            w *= 1000.0;
            w = w.floor(); // loses some of the precision from above but otherwise its ugly :mad:
            w /= 1000.0;

            // if w != 0.0 {
            weights.insert(stat, w);
            // }
        }
    }
    weights
}

fn parse_slot(s: &str) -> RelicSlot {
    match s {
        "PlanarSphere" => RelicSlot::Orb,
        "Hands" => RelicSlot::Hands,
        "Body" => RelicSlot::Body,
        "LinkRope" => RelicSlot::Rope,
        "Head" => RelicSlot::Head,
        "Feet" => RelicSlot::Feet,
        _ => panic!("unknown relic slot")
    }
}

fn parse_stat(s: &str) -> Option<RelicStat> {
    match s {
        "HP" => Some(RelicStat::Hp),
        "HP%" => Some(RelicStat::HpPercent),
        "ATK" => Some(RelicStat::Atk),
        "ATK%" => Some(RelicStat::AtkPercent),
        "DEF" => Some(RelicStat::Def),
        "DEF%" => Some(RelicStat::DefPercent),
        "CRIT Rate" => Some(RelicStat::CritRate),
        "CRIT DMG" => Some(RelicStat::CritDmg),
        "Break Effect" => Some(RelicStat::BreakEffect),
        "Effect Hit Rate" => Some(RelicStat::EffectHitRate),
        "Energy Regeneration Rate" => Some(RelicStat::EnergyRegenRate),
        "Fire DMG Boost" => Some(RelicStat::FireDmgBoost),
        "Ice DMG Boost" => Some(RelicStat::IceDmgBoost),
        "Imaginary DMG Boost" => Some(RelicStat::ImaginaryDmgBoost),
        "Lightning DMG Boost" => Some(RelicStat::LightningDmgBoost),
        "Outgoing Healing Boost" => Some(RelicStat::HealingBoost),
        "Physical DMG Boost" => Some(RelicStat::PhysDmgBoost),
        "Quantum DMG Boost" => Some(RelicStat::QuantumDmgBoost),
        "Effect RES" => Some(RelicStat::EffectRes),
        "SPD" => Some(RelicStat::Spd),
        "Wind DMG Boost" => Some(RelicStat::WindDmgBoost),
        _ => None
    }
}

// https://raw.githubusercontent.com/fribbels/hsr-optimizer/main/src/data/characters.json
// Object.values(ababa).map(({id, name}) => `${id} => Some("${name}")`).join(",\n")
fn parse_char_id(id: u32) -> Option<&'static str> {
    match id {
        1206 => Some("Sushang"),
        1202 => Some("Tingyun"),
        1201 => Some("Qingque"),
        1109 => Some("Hook"),
        1108 => Some("Sampo"),
        1106 => Some("Pela"),
        1105 => Some("Natasha"),
        1103 => Some("Serval"),
        1013 => Some("Herta"),
        1009 => Some("Asta"),
        1008 => Some("Arlan"),
        1002 => Some("Dan Heng"),
        1001 => Some("March 7th"),
        1207 => Some("Yukong"),
        1111 => Some("Luka"),
        1110 => Some("Lynx"),
        1210 => Some("Guinaifen"),
        1215 => Some("Hanya"),
        1214 => Some("Xueyi"),
        1312 => Some("Misha"),
        1301 => Some("Gallagher"),
        1224 => Some("March 7th - Hunt"),
        1223 => Some("Moze"),

        1211 => Some("Bailu"),
        1209 => Some("Yanqing"),
        1107 => Some("Clara"),
        1104 => Some("Gepard"),
        1004 => Some("Welt"),
        1003 => Some("Himeko"),
        1101 => Some("Bronya"),
        1102 => Some("Seele"),
        1204 => Some("Jing Yuan"),
        1006 => Some("Silver Wolf"),
        1203 => Some("Luocha"),
        1205 => Some("Blade"),
        1005 => Some("Kafka"),
        1213 => Some("Dan Heng • Imbibitor Lunae"),
        1208 => Some("Fu Xuan"),
        1212 => Some("Jingliu"),
        1112 => Some("Topaz & Numby"),
        1217 => Some("Huohuo"),
        1302 => Some("Argenti"),
        1303 => Some("Ruan Mei"),
        1305 => Some("Dr. Ratio"),
        1307 => Some("Black Swan"),
        1306 => Some("Sparkle"),
        1308 => Some("Acheron"),
        1304 => Some("Aventurine"),
        1309 => Some("Robin"),
        1315 => Some("Boothill"),
        1310 => Some("Firefly"),
        1314 => Some("Jade"),
        1221 => Some("Yunli"),
        1218 => Some("Jiaoqiu"),
        1220 => Some("Feixiao"),
        1222 => Some("Lingsha"),
        1317 => Some("Rappa"),
        1313 => Some("Sunday"),
        1225 => Some("Fugue"),
        1401 => Some("The Herta"),
        1402 => Some("Aglaea"),
        1403 => Some("Tribbie"),
        1404 => Some("Mydei"),
        1407 => Some("Castorice"),
        1405 => Some("Anaxa"),


        8001 => Some("Caelus (Destruction)"),
        8002 => Some("Stelle (Destruction)"),
        8003 => Some("Caelus (Preservation)"),
        8004 => Some("Stelle (Preservation)"),
        8005 => Some("Caelus (Harmony)"),
        8006 => Some("Stelle (Harmony)"),
        8007 => Some("Caelus (Remembrance)"),
        8008 => Some("Stelle (Remembrance)"),

        _ => None
    }
}

fn relic_score(relic: &Relic, weights: &HashMap<RelicStat, f64>) -> f64 {
    relic.subs.iter()
        .map(|r| weights.get(r).unwrap_or(&0f64) * 6.48 * 0.9)
        .sum::<f64>()
}