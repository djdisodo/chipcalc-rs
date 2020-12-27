use std::env;
use chipcalc_native_rust::calculation::{Board, CalculationJob, Config};
use std::str::FromStr;
use serde_json::Value;
use std::fs::File;
use chipcalc_native_rust::chip::Chip;
use chipcalc_native_rust::shape::Shape;
use chipcalc_native_rust::matrix::MatrixRotation;
use num_traits::cast::FromPrimitive;
use std::collections::{VecDeque, HashMap};
use std::iter::FromIterator;
use std::time::Instant;
use std::ops::Deref;


fn main() {
    let mut args = env::args();
    args.next(); //start path
    let file_path = args.next().unwrap();
    let board: Board = Board::from_str(&args.next().unwrap()).unwrap();
    let level: u8 = args.next().unwrap().parse().unwrap();
    let max_left_space: u8 = args.next().unwrap().parse().unwrap();
    let rotation: bool = args.next().unwrap().parse().unwrap();
    let allow_space: bool = args.next().unwrap().parse().unwrap();
    let min_rank: u8 = args.next().unwrap().parse().unwrap();
    let filter_color: u8 = args.next().unwrap().parse().unwrap();

    let canvas = board.to_canvas(level);
    let space = canvas.get_left_space();
    let data: Value = serde_json::from_reader(File::open(file_path).expect("파일 열기 실패")).expect("json 파싱 실패");
    let chip_datas = data["chip_with_user_info"].as_object().unwrap();
    let mut chip_ids: Vec<u32> = Vec::with_capacity(chip_datas.len());
    let mut chips: Vec<Chip> = Vec::with_capacity(chip_datas.len());
    for (key, x) in chip_datas {
        chip_ids.push(x["id"].as_str().unwrap().parse().unwrap());
        let a = x["shape_info"].as_str().unwrap().to_owned();
        let mut b = Shape::from_u32(x["grid_id"].as_str().unwrap().parse().unwrap()).unwrap();

        let rank: u8 = x["chip_id"].as_str().unwrap()[0..1].parse().unwrap();
        let color: u8 = x["color_id"].as_str().unwrap().parse().unwrap();
        if b.get_size() < min_rank as usize {
            continue;
        }
        if filter_color != color {
            continue;
        }
        let chip = Chip::from_json(x).unwrap();
        chips.push(chip);
    }
    println!("계산시작 칩갯수: {}", chips.len());
    for chip in &chips {
        println!("chip_shape: {:?}", chip.shape);
    }
    let job = CalculationJob::new(canvas, &chips, 0, Default::default(),  Config {
        min_chip_size: min_rank,
        rotate: rotation
    });
    let mut sub_jobs: Vec<CalculationJob> = Vec::new();
	for x in job.generate_jobs() {
		//let mut a: Vec<CalculationJob> = x.generate_jobs().collect();
		//sub_jobs.append(&mut a);
        sub_jobs.push(x);
	}
    println!("generated jobs {}", sub_jobs.len());

    let mut i = Instant::now();

	let all = sub_jobs.len();

    let mut done = 0;

    let mut l_done = 0;

    println!("space: {}", space);
	let mut d = 0;
    for x in sub_jobs {

        if Instant::now().duration_since(i).as_secs() > 1 {
            println!("done: {}", done);
            let time_per_rot = Instant::now().duration_since(i) / (done - l_done);
			println!("secs left: {}", (time_per_rot.as_millis() * (all - done as usize) as u128) / 1000);
            i = Instant::now();
            l_done = done;
        }
        done += 1;
        if let Some(result) = x.calculate() {
            for x in result {

                let mut left_size = space;
                for y in &*x {
                    left_size -= chips[y.chip_index].get_size() as u8;
                }
                if left_size <= max_left_space {
                    println!("----------------");
                    d += 1;
                    for chip in &*x {
                        println!("id: {}, pos: {} {}, rot: {:?}, chip_shape: {:?}", chip.chip_index, chip.position.x, chip.position.y, chip.rotation, chips[chip.chip_index].deref());
                    }
                }

            }
        }

    }
	println!("d: {}", d);
}
