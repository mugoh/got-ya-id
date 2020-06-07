//! String comparison

/// Calculates the Levenshtein distance between two strings
pub fn levenshtn(a: &str, b: &str) -> usize {
    let mut lev_dist = 0;

    if a == b {
        return lev_dist;
    }

    let length_a = a.chars().count();
    let length_b = b.chars().count();

    if length_a == 0 {
        return length_b;
    } else if length_b == 0 {
        return length_a;
    }

    let mut cache: Vec<usize> = vec![0; length_a];
    let mut idx_a = 0;
    let mut dist_a;
    let mut dist_b;

    while idx_a < length_a {
        idx_a += 1;
        cache[idx_a - 1] = idx_a;
    }

    for (idx_b, code_b) in b.chars().enumerate() {
        lev_dist = idx_b;
        dist_a = idx_b;

        for (idx_a, code_a) in a.chars().enumerate() {
            dist_b = if code_a == code_b { dist_a } else { dist_a + 1 };

            dist_a = cache[idx_a];

            lev_dist = if dist_a > lev_dist {
                if dist_b > lev_dist {
                    lev_dist + 1
                } else {
                    dist_b
                }
            } else if dist_b > dist_a {
                dist_a + 1
            } else {
                dist_b
            };

            cache[idx_a] = lev_dist;
        }
    }

    lev_dist
}
