use anyhow::{anyhow, Result};
use select::{document::Document, predicate::Name};
use std::{
    collections::{HashSet, VecDeque},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tokio::{
    fs,
    sync::{
        mpsc::{self, UnboundedReceiver},
        Semaphore,
    },
    try_join,
};
use tokio_util::task::TaskTracker;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let url = parse_args()?;
    let origin = url.origin().unicode_serialization();
    let domain = url
        .domain()
        .map(String::from)
        .ok_or(anyhow!("bad domain name"))?;

    visit_page(url, &origin, &domain).await?;

    Ok(())
}

fn parse_args() -> Result<Url> {
    let input = std::env::args().nth(1).ok_or(anyhow!("URL expected"))?;
    let url = input.parse()?;
    Ok(url)
}

async fn fs_worker(mut rx: UnboundedReceiver<(Url, PathBuf)>) {
    let tracker = TaskTracker::new();
    let sem = Arc::new(Semaphore::new(10));

    while let Some((url, path)) = rx.recv().await {
        let sem_ = sem.clone();

        tracker.spawn(async move {
            let permit = sem_.acquire().await.unwrap();
            if let Err(why) = download_from_to(url, path).await {
                println!("[err]: failed downloading by path: {}", why);
            }
            drop(permit);
        });
    }

    tracker.close();
    tracker.wait().await;

    async fn download_from_to(url: Url, path: PathBuf) -> Result<()> {
        println!(
            " - '{}' --> '{}'..",
            url.as_str(),
            path.to_str().unwrap_or("<unknown>")
        );

        let get_content = async {
            let content = reqwest::get(url).await?.bytes().await?;
            anyhow::Ok(content)
        };

        let create_file = async {
            let parent = path.parent().ok_or(anyhow!("cannot get parent"))?;
            fs::create_dir_all(parent).await?;
            anyhow::Ok(File::create(path).unwrap())
        };

        let (bytes, mut file) = try_join!(get_content, create_file)?;
        file.write_all(&bytes)?;

        Ok(())
    }
}

async fn visit_page(url: Url, origin: &str, domain: &str) -> Result<()> {
    let get_full_path = |url: &Url| {
        let mut path = Path::new(url.path().trim_start_matches(['/', '#']));
        if path == Path::new("") {
            path = Path::new("index.html");
        }
        let mut full_path = Path::new(domain).join(path);
        if full_path.extension().is_none() {
            full_path.set_extension("html");
        }
        full_path
    };

    let (tx, rx) = mpsc::unbounded_channel();
    let worker = tokio::spawn(fs_worker(rx));

    let full_path = get_full_path(&url);
    tx.send((url.clone(), full_path))?;

    let mut urls = VecDeque::from([url]);
    let mut visited = HashSet::new();

    while let Some(url) = urls.pop_front() {
        println!("visiting '{}'", url);
        visited.insert(url.clone());

        let content = reqwest::get(url).await?.text().await?;
        let document = Document::from(content.as_str());

        let hrefs = document.find(Name("a")).filter_map(|n| n.attr("href"));
        let imgs = document.find(Name("img")).filter_map(|n| n.attr("src"));
        let scripts = document.find(Name("script")).filter_map(|n| n.attr("src"));

        for str in hrefs
            .chain(imgs)
            .chain(scripts)
            .filter(|str| Url::parse(str).as_ref().map(Url::origin).ok().is_none())
        {
            let url = Url::from_str(origin)?.join(str)?;
            println!("  to visit: {}", url);

            if visited.contains(&url) {
                println!("  ..but visited");
                continue;
            }

            if Path::new(str).extension().is_none() {
                println!("  keep to visit: {}", str);
                urls.push_back(url.clone());
            }

            let full_path = get_full_path(&url);
            tx.send((url.clone(), full_path))?;
        }
    }

    drop(tx);
    worker.await?;

    Ok(())
}
