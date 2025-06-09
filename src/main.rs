use std::fs; // File management
use std::path::Path; // File path management
use std::env; // Runtime environment
use std::io::{self, Write}; // Input/Output
use std::time::{SystemTime, UNIX_EPOCH}; // Time management

struct CleanupStats { // Cleanup statistics
    files_deleted: u64, // Number of deleted files
    bytes_freed: u64, // Bytes freed
    errors: u64, // Number of errors
}

impl CleanupStats {
    fn new() -> Self {
        CleanupStats { // Initialize everything to 0
            files_deleted: 0,
            bytes_freed: 0,
            errors: 0,
        }
    }

    fn add_file(&mut self, size: u64) { // When a file is deleted, update stats
        self.files_deleted += 1;
        self.bytes_freed += size;
    }

    fn add_error(&mut self) { // Same if there's an error
        self.errors += 1;
    }
}

fn main() {
    println!("Windows System Cleaner");
    println!("========================");
    println!("This tool will clean temporary files and browser caches safely.");
    println!("Personal files and important data will NOT be touched.\n");

    loop {
        print!("Continue? (y/n): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() == "y" {
            break;
        }
        else if input.trim().to_lowercase() == "n" {
            println!("Operation cancelled.");
            return;
        }
        else {
            println!("You need to say yes or no, not some random shit");
        }
    }

    let mut total_stats = CleanupStats::new(); // New instance for stats

    println!("\nStarting cleanup...\n");
    // Call the functions below
    // Clean Windows temp directories
    clean_windows_temp(&mut total_stats);

    // Clean browser caches
    clean_browser_caches(&mut total_stats);

    // Clean Windows prefetch (optional)
    clean_prefetch(&mut total_stats);

    // Clean Windows thumbnail cache
    clean_thumbnail_cache(&mut total_stats);

    // Display results
    println!("\nCleanup completed!");
    println!("===================");
    println!("Files deleted: {}", total_stats.files_deleted);
    println!("Space freed: {:.2} MB", total_stats.bytes_freed as f64 / 1024.0 / 1024.0);
    if total_stats.errors > 0 {
        println!("Errors encountered: {} (some files may be in use)", total_stats.errors);
    }

    println!("\nPress Enter to exit...");
    let mut _input = String::new();
    io::stdin().read_line(&mut _input).unwrap();
}

fn clean_windows_temp(stats: &mut CleanupStats) {
    println!("Cleaning Windows temp directories...");

    // %TEMP% directory
    if let Ok(temp_dir) = env::var("TEMP") {
        clean_directory(&temp_dir, stats, false);
    }

    // %TMP% directory (usually same as TEMP)
    if let Ok(tmp_dir) = env::var("TMP") {
        clean_directory(&tmp_dir, stats, false);
    }

    // Windows temp directory
    let windows_temp = r"C:\Windows\Temp";
    if Path::new(windows_temp).exists() {
        clean_directory(windows_temp, stats, false);
    }
}

fn clean_browser_caches(stats: &mut CleanupStats) {
    println!("Cleaning browser caches...");

    if let Ok(appdata) = env::var("LOCALAPPDATA") {
        let browsers = vec![
            // Chrome
            (format!("{}/Google/Chrome/User Data/Default/Cache", appdata), "Chrome Cache"),
            (format!("{}/Google/Chrome/User Data/Default/Code Cache", appdata), "Chrome Code Cache"),

            // Edge
            (format!("{}/Microsoft/Edge/User Data/Default/Cache", appdata), "Edge Cache"),
            (format!("{}/Microsoft/Edge/User Data/Default/Code Cache", appdata), "Edge Code Cache"),

            // Brave
            (format!("{}/BraveSoftware/Brave-Browser/User Data/Default/Cache", appdata), "Brave Cache"),
            (format!("{}/BraveSoftware/Brave-Browser/User Data/Default/Code Cache", appdata), "Brave Code Cache"),

            // Opera
            (format!("{}/Opera Software/Opera Stable/Cache", appdata), "Opera Cache"),
        ];

        for (path, name) in browsers {
            if Path::new(&path).exists() {
                println!("  Cleaning {}...", name);
                clean_directory(&path, stats, true);
            }
        }
    }

    // Firefox cache (different location)
    if let Ok(appdata) = env::var("LOCALAPPDATA") {
        let firefox_cache = format!("{}/Mozilla/Firefox/Profiles", appdata);
        if Path::new(&firefox_cache).exists() {
            clean_firefox_cache(&firefox_cache, stats);
        }
    }
}

fn clean_firefox_cache(profiles_dir: &str, stats: &mut CleanupStats) {
    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let cache_path = entry.path().join("cache2");
                if cache_path.exists() {
                    println!("  Cleaning Firefox Cache...");
                    clean_directory(&cache_path.to_string_lossy(), stats, true);
                }
            }
        }
    }
}

fn clean_prefetch(stats: &mut CleanupStats) {
    println!("Cleaning Windows prefetch...");
    let prefetch_dir = r"C:\Windows\Prefetch";

    if Path::new(prefetch_dir).exists() {
        // Only clean prefetch files older than 30 days
        clean_directory_with_age_filter(prefetch_dir, stats, 30);
    }
}

fn clean_thumbnail_cache(stats: &mut CleanupStats) {
    println!("Cleaning thumbnail cache...");

    if let Ok(appdata) = env::var("LOCALAPPDATA") {
        let thumb_cache = format!("{}/Microsoft/Windows/Explorer", appdata);
        if Path::new(&thumb_cache).exists() {
            clean_directory(&thumb_cache, stats, false);
        }
    }
}

fn clean_directory(dir_path: &str, stats: &mut CleanupStats, recursive: bool) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                // Get file size before deletion
                if let Ok(metadata) = fs::metadata(&path) {
                    let size = metadata.len();

                    // Try to delete the file
                    match fs::remove_file(&path) {
                        Ok(_) => stats.add_file(size),
                        Err(_) => stats.add_error(), // File might be in use
                    }
                }
            } else if path.is_dir() && recursive {
                clean_directory(&path.to_string_lossy(), stats, recursive);
                // Try to remove empty directory
                let _ = fs::remove_dir(&path);
            }
        }
    }
}

fn clean_directory_with_age_filter(dir_path: &str, stats: &mut CleanupStats, max_age_days: u64) {
    let max_age_secs = max_age_days * 24 * 60 * 60;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        let modified_secs = modified
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        // Only delete files older than max_age_days
                        if now - modified_secs > max_age_secs {
                            let size = metadata.len();
                            match fs::remove_file(&path) {
                                Ok(_) => stats.add_file(size),
                                Err(_) => stats.add_error(),
                            }
                        }
                    }
                }
            }
        }
    }
}

//Development test, not used in the cargo build version
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_stats() {
        let mut stats = CleanupStats::new();
        stats.add_file(1000);
        stats.add_file(2000);
        stats.add_error();

        assert_eq!(stats.files_deleted, 2);
        assert_eq!(stats.bytes_freed, 3000);
        assert_eq!(stats.errors, 1);
    }
}