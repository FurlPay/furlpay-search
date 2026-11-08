//! Robots.txt policy parser and enforcement for web crawling.

use std::collections::HashMap;

/// Rule for a specific user-agent path prefix.
#[derive(Debug, Clone)]
pub struct RobotsRule {
    pub user_agent: String,
    pub disallowed_paths: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub crawl_delay_secs: Option<u64>,
}

/// Parsed robots.txt representation for a domain.
#[derive(Debug, Clone, Default)]
pub struct RobotsPolicy {
    /// Rules indexed by user-agent (e.g. "*", "furlpaybot").
    pub rules: HashMap<String, RobotsRule>,
}

impl RobotsPolicy {
    /// Parse robots.txt file content.
    pub fn parse(content: &str) -> Self {
        let mut policy = RobotsPolicy::default();
        let mut current_agents: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((directive, value)) = line.split_once(':') {
                let directive = directive.trim().to_lowercase();
                let value = value.trim();

                match directive.as_str() {
                    "user-agent" => {
                        let agent = value.to_lowercase();
                        current_agents.push(agent);
                    }
                    "disallow" => {
                        for agent in &current_agents {
                            let entry = policy.rules.entry(agent.clone()).or_insert_with(|| RobotsRule {
                                user_agent: agent.clone(),
                                disallowed_paths: Vec::new(),
                                allowed_paths: Vec::new(),
                                crawl_delay_secs: None,
                            });
                            if !value.is_empty() {
                                entry.disallowed_paths.push(value.to_string());
                            }
                        }
                    }
                    "allow" => {
                        for agent in &current_agents {
                            let entry = policy.rules.entry(agent.clone()).or_insert_with(|| RobotsRule {
                                user_agent: agent.clone(),
                                disallowed_paths: Vec::new(),
                                allowed_paths: Vec::new(),
                                crawl_delay_secs: None,
                            });
                            if !value.is_empty() {
                                entry.allowed_paths.push(value.to_string());
                            }
                        }
                    }
                    "crawl-delay" => {
                        if let Ok(delay) = value.parse::<u64>() {
                            for agent in &current_agents {
                                let entry = policy.rules.entry(agent.clone()).or_insert_with(|| RobotsRule {
                                    user_agent: agent.clone(),
                                    disallowed_paths: Vec::new(),
                                    allowed_paths: Vec::new(),
                                    crawl_delay_secs: None,
                                });
                                entry.crawl_delay_secs = Some(delay);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        policy
    }

    /// Check if a URL path is allowed for the given user-agent.
    pub fn is_allowed(&self, user_agent: &str, path: &str) -> bool {
        let agent_lower = user_agent.to_lowercase();

        // Check specific agent rule first, then wildcard "*"
        let rule = self.rules.get(&agent_lower).or_else(|| self.rules.get("*"));

        if let Some(rule) = rule {
            // Longest matching prefix rule wins (standard RFC 9309)
            let mut longest_allowed = 0;
            for allowed in &rule.allowed_paths {
                if path.starts_with(allowed) && allowed.len() > longest_allowed {
                    longest_allowed = allowed.len();
                }
            }

            let mut longest_disallowed = 0;
            for disallowed in &rule.disallowed_paths {
                if path.starts_with(disallowed) && disallowed.len() > longest_disallowed {
                    longest_disallowed = disallowed.len();
                }
            }

            if longest_disallowed > longest_allowed {
                return false;
            }
        }

        true
    }

    /// Get crawl delay in seconds, if specified.
    pub fn get_crawl_delay(&self, user_agent: &str) -> Option<u64> {
        let agent_lower = user_agent.to_lowercase();
        self.rules.get(&agent_lower)
            .or_else(|| self.rules.get("*"))
            .and_then(|r| r.crawl_delay_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_robots_txt() {
        let robots = "
User-agent: *
Disallow: /admin/
Disallow: /private/
Allow: /admin/public/
Crawl-delay: 2

User-agent: Googlebot
Disallow: /nogoogle/
";
        let policy = RobotsPolicy::parse(robots);

        assert!(!policy.is_allowed("furlpaybot", "/admin/dashboard"));
        assert!(policy.is_allowed("furlpaybot", "/admin/public/page"));
        assert!(!policy.is_allowed("furlpaybot", "/private/data"));
        assert!(policy.is_allowed("furlpaybot", "/blog/post-1"));
        assert_eq!(policy.get_crawl_delay("furlpaybot"), Some(2));

        assert!(!policy.is_allowed("Googlebot", "/nogoogle/secret"));
    }
}
