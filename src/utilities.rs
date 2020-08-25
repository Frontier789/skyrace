use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub fn watch(paths: Vec<&str>) -> notify::Result<(RecommendedWatcher, Receiver<DebouncedEvent>)> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs_f32(0.2))?;

    for p in paths.into_iter() {
        watcher.watch(p, RecursiveMode::NonRecursive)?;
    }

    Ok((watcher, rx))
}
