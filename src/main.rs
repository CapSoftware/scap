use cypher;

fn main() {
    // #1 Check if the platform is supported
    let supported = cypher::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // #2 Check if the app has permission to capture the screen
    let has_permission = cypher::has_permission();
    if !has_permission {
        println!("❌ Permission not granted");
        return;
    } else {
        println!("✅ Permission granted");
    }

    // #3 Capture the screen
    cypher::capture();
}
