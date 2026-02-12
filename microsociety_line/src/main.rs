use microsociety_line::run::{demo_world, Episode};

fn main() {
    // Build a small 1-D micro-society.
    let world = demo_world(51);
    let mut episode = Episode::new("demo_sharing_vs_conflict", world);

    // Run for a number of ticks to observe colonization and trust dynamics.
    episode.run_for_ticks(200);

    // Save results as a knowledge_object-like JSON file for later reflection.
    if let Err(e) = episode.save_json("episode_demo_sharing_vs_conflict.json") {
        eprintln!("Failed to save episode: {e}");
    } else {
        println!("Episode saved to episode_demo_sharing_vs_conflict.json");
    }
}
