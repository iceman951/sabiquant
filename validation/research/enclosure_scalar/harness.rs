//! Private stdin/stdout adapter. Allocation and I/O are confined to this harness.
#![forbid(unsafe_code)]
use bounded_scalar::{Frame, Point, Work, ZERO_FRAME, ZERO_POINT};
use std::io::{self, BufRead, Write};

fn words(dst: &mut [u64], text: &str) -> Result<(), String> {
    dst.fill(0);
    for (i, part) in text.split(',').enumerate() {
        let slot = dst.get_mut(i).ok_or("too many input limbs")?;
        *slot = u64::from_str_radix(part, 16).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn load(out: &mut Point, fields: &[&str]) -> Result<(), String> {
    words(&mut out.m, fields[1])?;
    words(&mut out.k, fields[3])?;
    out.flags = u64::from(fields[0] == "-1") | (u64::from(fields[2] == "-1") << 1);
    Ok(())
}
fn encode(words: &[u64]) -> String {
    let len = words.iter().rposition(|&x| x != 0).map_or(1, |n| n + 1);
    words[..len]
        .iter()
        .map(|x| format!("{x:016x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn encoded_point(p: &Point) -> String {
    let sign = if p.m.iter().all(|&x| x == 0) {
        0
    } else if p.flags & 1 != 0 {
        -1
    } else {
        1
    };
    let ksign = if p.k.iter().all(|&x| x == 0) {
        0
    } else if p.flags & 2 != 0 {
        -1
    } else {
        1
    };
    format!("{sign}\t{}\t{ksign}\t{}", encode(&p.m), encode(&p.k))
}
fn main() -> Result<(), String> {
    let input = io::stdin();
    let output = io::stdout();
    let mut output = io::BufWriter::new(output.lock());
    writeln!(output, "id\top\tstatus\tlower_sign\tlower_m_words\tlower_K_sign\tlower_K_words\tupper_sign\tupper_m_words\tupper_K_sign\tupper_K_words\tlower_binary64_bits\tupper_binary64_bits\twork_used").map_err(|e| e.to_string())?;
    let mut frame: Frame = ZERO_FRAME;
    let mut a = ZERO_POINT;
    let mut b = ZERO_POINT;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    for (index, line) in input.lock().lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if index == 0 {
            continue;
        }
        let f: Vec<_> = line.split('\t').collect();
        if f.len() != 20 {
            return Err(format!("row{index}: expected20columns"));
        }
        let p = f[2].parse().map_err(|_| "bad precision")?;
        let mut work = Work {
            used: 0,
            max: u64::MAX,
        };
        let mut lower_bits = String::from("_");
        let mut upper_bits = String::from("_");
        let result = match f[1] {
            "normalize" => {
                words(&mut frame.raw0.n, f[4])?;
                words(&mut frame.raw0.d, f[5])?;
                words(&mut frame.e0, f[7])?;
                bounded_scalar::normalize(
                    p,
                    f[3] == "1",
                    f[6] == "-1",
                    &mut lo,
                    &mut hi,
                    &mut frame,
                    &mut work,
                )
            }
            "mul" | "div" | "add" => {
                load(&mut a, &f[8..12])?;
                load(&mut b, &f[12..16])?;
                match f[1] {
                    "mul" => {
                        bounded_scalar::multiply(p, &a, &b, &mut lo, &mut hi, &mut frame, &mut work)
                    }
                    "div" => {
                        bounded_scalar::divide(p, &a, &b, &mut lo, &mut hi, &mut frame, &mut work)
                    }
                    _ => bounded_scalar::add(p, &a, &b, &mut lo, &mut hi, &mut frame, &mut work),
                }
            }
            "export_binary64" => {
                load(&mut a, &f[16..20])?;
                match bounded_scalar::export(p, &a, &mut work) {
                    Ok(out) => {
                        lower_bits = out
                            .lower
                            .map_or_else(|| "unavailable".to_string(), |x| format!("{x:016x}"));
                        upper_bits = out
                            .upper
                            .map_or_else(|| "unavailable".to_string(), |x| format!("{x:016x}"));
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            other => return Err(format!("unsupported op {other}")),
        };
        let status = match result {
            Ok(()) => "ok",
            Err(bounded_scalar::Error::Precision) => "precision",
            Err(bounded_scalar::Error::DenominatorZero) => "zero_denominator",
            Err(bounded_scalar::Error::Capacity) => "capacity",
            Err(bounded_scalar::Error::Malformed) => "malformed",
            Err(bounded_scalar::Error::WorkLimit) => "work_limit",
        };
        let lower = if status == "ok" && f[1] != "export_binary64" {
            encoded_point(&lo)
        } else {
            "_\t_\t_\t_".into()
        };
        let upper = if status == "ok" && f[1] != "export_binary64" {
            encoded_point(&hi)
        } else {
            "_\t_\t_\t_".into()
        };
        writeln!(
            output,
            "{}\t{}\t{status}\t{lower}\t{upper}\t{lower_bits}\t{upper_bits}\t{}",
            f[0], f[1], work.used
        )
        .map_err(|e| e.to_string())?;
    }
    output.flush().map_err(|e| e.to_string())?;
    Ok(())
}
