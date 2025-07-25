pub mod client;
pub mod server;

struct Env {
    token: &'static str,
    pubkey: &'static str,
    appid: &'static str,
}

const fn find_until(slice: &[u8], target: u8) -> Option<usize> {
    let mut i = 0;
    loop {
        if i == slice.len() {
            return None;
        }

        if slice[i] == target {
            return Some(i);
        }

        i += 1;
    }
}

const ENV: Env = {
    const DOTENV: &[u8] = include_bytes!("../../.env");
    let mut token = None;
    let mut pubkey = None;
    let mut appid = None;

    let mut i = 0;
    let mut left = DOTENV;
    loop {
        if token.is_some() && pubkey.is_some() && appid.is_some() {
            break;
        }

        if i == left.len() {
            panic!();
        }

        if left[i] == b'=' {
            let (name, remainder) = left.split_at(i);
            match name {
                b"DISCORD_TOKEN" => {
                    let (_, remainder) = remainder.split_first().unwrap();
                    let (raw_token, remainder) =
                        remainder.split_at(find_until(remainder, b'\n').unwrap());
                    match str::from_utf8(raw_token) {
                        Ok(t) => {
                            token = Some(t);
                        }
                        Err(_) => panic!(),
                    }
                    let (_, remainder) = remainder.split_first().unwrap();
                    left = remainder;
                    i = 0;
                    continue;
                }
                b"PUBLIC_KEY" => {
                    let (_, remainder) = remainder.split_first().unwrap();
                    let (raw_pubkey, remainder) =
                        remainder.split_at(find_until(remainder, b'\n').unwrap());
                    match str::from_utf8(raw_pubkey) {
                        Ok(k) => {
                            pubkey = Some(k);
                        }
                        Err(_) => panic!(),
                    }
                    let (_, remainder) = remainder.split_first().unwrap();
                    left = remainder;
                    i = 0;
                    continue;
                }
                b"APP_ID" => {
                    let (_, remainder) = remainder.split_first().unwrap();
                    let (raw_appid, remainder) =
                        remainder.split_at(find_until(remainder, b'\n').unwrap());
                    match str::from_utf8(raw_appid) {
                        Ok(k) => {
                            appid = Some(k);
                        }
                        Err(_) => panic!(),
                    }
                    let (_, remainder) = remainder.split_first().unwrap();
                    left = remainder;
                    i = 0;
                    continue;
                }
                _ => panic!(),
            }
        }

        i += 1;
    }

    Env {
        token: token.unwrap(),
        pubkey: pubkey.unwrap(),
        appid: appid.unwrap(),
    }
};
