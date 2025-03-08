use std::collections::{HashMap, HashSet};

pub fn group_by_common_full_words(groups: Vec<Vec<String>>) -> Vec<(String, Vec<String>)> {
    let mut result = Vec::new();

    for group in groups {
        if group.is_empty() {
            continue;
        }

        // If the group has only one element, use the whole string as the common part
        if group.len() == 1 {
            result.push((group[0].clone(), group.clone()));
            continue;
        }

        // Track all processed strings to avoid duplicates
        let mut processed_strings = HashSet::new();

        // Create a graph of strings that share common words
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // For each pair of strings in the group
        for i in 0..group.len() {
            for j in i + 1..group.len() {
                let s1 = &group[i];
                let s2 = &group[j];

                // Check if they share a common word
                if !has_common_word(s1, s2).is_empty() {
                    graph.entry(s1.clone()).or_default().push(s2.clone());
                    graph.entry(s2.clone()).or_default().push(s1.clone());
                }
            }
        }

        // Find connected components (subgroups)
        let mut visited = HashSet::new();

        for string in &group {
            if visited.contains(string) {
                continue;
            }

            // Find all connected strings (DFS)
            let mut subgroup = Vec::new();
            let mut stack = vec![string.clone()];

            while let Some(current) = stack.pop() {
                if visited.insert(current.clone()) {
                    subgroup.push(current.clone());

                    if let Some(neighbors) = graph.get(&current) {
                        for neighbor in neighbors {
                            if !visited.contains(neighbor) {
                                stack.push(neighbor.clone());
                            }
                        }
                    }
                }
            }

            // Process each subgroup
            if !subgroup.is_empty() {
                // Mark all strings in this subgroup as processed
                for s in &subgroup {
                    processed_strings.insert(s.clone());
                }
                process_subgroup(&subgroup, &mut result);
            }
        }

        // Process individual items that had no connections and haven't been processed yet
        for string in &group {
            if !graph.contains_key(string) && !processed_strings.contains(string) {
                result.push((string.clone(), vec![string.clone()]));
                processed_strings.insert(string.clone());
            }
        }
    }

    // If there are groups with the same name, add a number suffix to the name
    let mut name_counts = HashMap::new();
    for (name, _) in result.iter_mut() {
        let count = name_counts.entry(name.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            *name = format!("{}_{}", name, count);
        }
    }

    result
}

// Helper function to check if two strings share a common word
fn has_common_word(s1: &str, s2: &str) -> Vec<String> {
    let parts1: Vec<&str> = s1.split('_').collect();
    let parts2: Vec<&str> = s2.split('_').collect();

    // Check for common suffix (right side)
    for i in 1..=std::cmp::min(parts1.len(), parts2.len()) {
        if parts1[parts1.len() - i] == parts2[parts2.len() - i] {
            return vec![parts1[parts1.len() - i].to_string()];
        }
    }

    // Check for common prefix (left side)
    for i in 0..std::cmp::min(parts1.len(), parts2.len()) {
        if parts1[i] == parts2[i] {
            return vec![parts1[i].to_string()];
        }

        // Break after first mismatch on prefix
        if parts1[i] != parts2[i] {
            break;
        }
    }

    vec![]
}

// Process a subgroup of strings that share common words
fn process_subgroup(subgroup: &[String], result: &mut Vec<(String, Vec<String>)>) {
    if subgroup.len() == 1 {
        result.push((subgroup[0].clone(), subgroup.to_vec()));
        return;
    }

    // Create a mutable copy of the subgroup so we can remove items as we process them
    let mut remaining_strings: Vec<String> = subgroup.to_vec();
    println!("remaining_strings: {:?}", remaining_strings);

    while !remaining_strings.is_empty() {
        let current = remaining_strings[0].clone();
        let mut current_group = vec![current.clone()];
        let mut common_word = String::new();

        // Find all strings that share common words with the current string
        let mut i = 1;
        while i < remaining_strings.len() {
            let common_words = has_common_word(&current, &remaining_strings[i]);
            if !common_words.is_empty() {
                if common_word.is_empty() {
                    common_word = common_words[0].clone();
                    println!("common_word: {:?}", common_word);
                }
                if common_word == common_words[0] {
                    current_group.push(remaining_strings[i].clone());
                    remaining_strings.remove(i);
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Remove the current string
        remaining_strings.remove(0);

        // Find the best common part for this group
        find_best_common_part(&current_group, result);
    }
}

// Helper function to find the best common part among a group of strings
fn find_best_common_part(strings: &[String], result: &mut Vec<(String, Vec<String>)>) {
    if strings.len() == 1 {
        result.push((strings[0].clone(), strings.to_vec()));
        return;
    }

    // Find all possible full words (delimited by underscores) in each string
    let mut all_possible_common_words = HashMap::new();

    for string in strings {
        let parts: Vec<&str> = string.split('_').collect();

        // Get all possible combinations of consecutive parts
        for start in 0..parts.len() {
            for end in start + 1..=parts.len() {
                let common_part = parts[start..end].join("_");

                // Make sure it's a full word/phrase (starts or ends with underscore)
                let is_full_word = start == 0 || end == parts.len();

                if is_full_word {
                    all_possible_common_words
                        .entry(common_part)
                        .or_insert_with(HashSet::new)
                        .insert(string.clone());
                }
            }
        }
    }

    // Find the best common word/phrase
    let mut best_common_part = String::new();
    let mut best_score = -1;
    let mut best_matches = HashSet::new();

    for (common_part, matching_strings) in all_possible_common_words {
        // Only consider if it matches all strings in the subgroup
        if matching_strings.len() == strings.len() {
            // Calculate score based on length and position
            let score = (common_part.len() * 10) as i32;

            if score > best_score
                || (score == best_score && common_part.len() > best_common_part.len())
            {
                best_score = score;
                best_common_part = common_part;
                best_matches = matching_strings;
            }
        }
    }

    // For cases where we couldn't find a common full word/phrase
    // Fallback to finding the common suffix (from the right)
    if best_common_part.is_empty() {
        let words: Vec<Vec<&str>> = strings.iter().map(|s| s.split('_').collect()).collect();

        'outer: for i in 1..=words.iter().map(|w| w.len()).min().unwrap_or(0) {
            let suffix = words[0][words[0].len() - i];

            for word_parts in &words[1..] {
                if word_parts[word_parts.len() - i] != suffix {
                    continue 'outer;
                }
            }

            // Found a common suffix
            best_common_part = suffix.to_string();
            best_matches = strings.iter().cloned().collect();
            break;
        }
    }

    // Add to results
    if !best_common_part.is_empty() {
        result.push((best_common_part, best_matches.into_iter().collect()));
    } else {
        // If still no common parts, handle each string individually
        for string in strings {
            result.push((string.clone(), vec![string.clone()]));
        }
    }
}
