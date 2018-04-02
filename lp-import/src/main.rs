extern crate diesel;
extern crate glob;
extern crate lp;
extern crate lp_import;
extern crate lp_magick;
extern crate toml;
extern crate futures;
extern crate futures_cpupool;
extern crate rand;

use diesel::prelude::*;
use futures::Future;
use futures_cpupool::CpuPool;
use glob::glob;
use lp::models::ReleaseId;
use lp_magick::resize;
use rand::{thread_rng, Rng};
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use toml::Value;

use lp_import::{Context, parameterize, readers};

static ARTIST_KINDS: [&'static str; 2] = ["people", "groups"];

struct HexGenerator<'a, R: 'a> {
    rng: &'a mut R,
}

impl<'a, R: Rng> Iterator for HexGenerator<'a, R> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        const HEX_CHARSET: &'static [u8] = b"0123456789abcdef";
        Some(*self.rng.choose(HEX_CHARSET).unwrap() as char)
    }
}

fn generate_id() -> String {
    let mut rng = thread_rng();
    let generator = HexGenerator { rng: &mut rng };
    generator.take(32).collect()
}

fn read_toml<F>(pattern: &str, mut callback: F)
    where F: FnMut(&Path, Value),
{
    let entries = glob(pattern)
        .expect("bad glob pattern")
        .filter_map(Result::ok);

    for entry in entries {
        let mut file = File::open(&entry).expect("could not open file");
        let mut toml = String::new();
        file.read_to_string(&mut toml).expect("could not read file");

        let data = toml.parse().unwrap();
        callback(&entry, data);
    }
}

fn medium_kind_to_label(kind: i32) -> &'static str {
    match kind {
        0 => "cd",
        1 => "dvd",
        2 => "blu-ray",
        3 => "digital",
        4 => "vinyl",
        _ => unreachable!(),
    }
}

fn update_release_artwork_data(release_id: ReleaseId,
                               original_id: &str,
                               thumbnail_id: &str)
{
    use lp::schema::releases::dsl::*;

    let ctx = Context::new();

    let data = format!(
        r#"{{"original":{{"id":"{}"}},"thumbnail":{{"id":"{}"}}}}"#,
        original_id,
        thumbnail_id,
    );

    diesel::update(releases.find(release_id))
        .set(artwork_data.eq(&data))
        .execute(ctx.connection())
        .unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let pathname = args.get(1).expect("missing working directory");
    let store_dir = args.get(2).cloned();

    if let Some(ref dst) = store_dir {
        fs::create_dir_all(dst).expect("failed to create store_dir");
    }

    let pool = CpuPool::new_num_cpus();
    let mut tasks = Vec::new();

    let mut ctx = Context::new();
    let suffix = ".toml";

    // artists

    for kind in &ARTIST_KINDS {
        let prefix = format!("{}/artists/{}/", pathname, kind);
        let pattern = format!("{}**/*{}", prefix, suffix);

        read_toml(&pattern, |path, value| {
            let path = path.to_str().unwrap();
            let start = prefix.len();
            let end = path.len() - suffix.len();
            let id = &path[start..end];

            let artist = match readers::artist::create(&ctx, &value) {
                Ok(a) => a,
                Err(e) => panic!("{}: {:?}", path, e),
            };

            ctx.artists.insert(String::from(id), artist);
        });
    }

    // albums

    let prefix = format!("{}/albums/", pathname);
    let start = prefix.len();
    let pattern = format!("{}**/*{}", prefix, suffix);

    read_toml(&pattern, |path, value| {
        let path = path.to_str().unwrap();
        let end = path.len() - suffix.len();
        let id = &path[start..end];

        let (_, releases) = match readers::album::create(&ctx, &value) {
            Ok(a) => a,
            Err(e) => panic!("{}: {:?}", path, e),
        };

        for (release, media) in releases {
            let disambiguation = if let Some(ref d) = release.disambiguation {
                parameterize(d)
            } else {
                String::from("default")
            };

            let first_format = medium_kind_to_label(media[0].kind);

            for medium in media {
                let medium_kind = medium_kind_to_label(medium.kind);
                let medium_id = format!("{}/{}/{}{}", id, disambiguation, medium_kind, medium.position);
                ctx.media.insert(medium_id, medium);
            }

            if let Some(ref dst_prefix) = store_dir {
                let id = id.to_string();
                let release_id = release.id;
                let dst_prefix = dst_prefix.clone();
                let pathname = pathname.to_string();
                let disambiguation = disambiguation.to_string();

                let original_id = generate_id();
                let thumbnail_id = generate_id();

                let task = pool.spawn_fn(move || {
                    let res: Result<(), ()> = Ok(());
                    let mut src = format!("{}/-attachments/albums/{}/{}-{}.jpg", pathname, id, disambiguation, first_format);

                    if !Path::new(&src).exists() {
                        src = format!("{}/-attachments/albums/{}/{}.jpg", pathname, id, disambiguation);

                        if !Path::new(&src).exists() {
                            panic!("missing artwork: {}", src);
                        }
                    }

                    let dst = format!("{}/{}.jpg", dst_prefix, original_id);
                    fs::copy(&src, &dst).expect("artwork copy failed");

                    let dst = format!("{}/{}.jpg", dst_prefix, thumbnail_id);
                    resize(&src, &dst, 256, 256);

                    update_release_artwork_data(release_id, &original_id, &thumbnail_id);

                    res
                });

                tasks.push(task);
            }
        }
    });

    // songs

    let prefix = format!("{}/songs/", pathname);
    let start = prefix.len();
    let pattern = format!("{}**/*{}", prefix, suffix);

    read_toml(&pattern, |path, value| {
        let mut p = path.to_path_buf();
        p.pop();
        let p = p.to_str().unwrap();
        let artist_id = &p[start..];

        let path = path.to_str().unwrap();
        let end = path.len() - suffix.len();
        let id = &path[start..end];

        let song = match readers::song::create(&ctx, &value, artist_id) {
            Ok(s) => s,
            Err(e) => panic!("{}: {:?}", path, e),
        };

        ctx.songs.insert(String::from(id), song);
    });

    // tracklists

    let pattern = format!("{}/tracklists/**/*.toml", pathname);

    read_toml(&pattern, |path, value| {
        if let Err(e) = readers::tracklist::create(&ctx, &value) {
            let path = path.to_str().unwrap();
            panic!("{}: {:?}", path, e);
        }
    });

    // attachments

    for task in tasks {
        task.wait().unwrap();
    }
}
