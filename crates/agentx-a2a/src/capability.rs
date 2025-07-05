//! Capability Discovery and Management
//! 
//! This module provides functionality for capability discovery,
//! matching, and management in the A2A protocol.

use crate::{AgentCard, Capability, CapabilityType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capability query for discovering agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityQuery {
    /// Required capabilities
    pub required: Vec<CapabilityRequirement>,
    
    /// Optional capabilities (nice to have)
    #[serde(default)]
    pub optional: Vec<CapabilityRequirement>,
    
    /// Query filters
    #[serde(default)]
    pub filters: QueryFilters,
    
    /// Maximum number of results
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

/// Capability requirement specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityRequirement {
    /// Capability name or pattern
    pub name: String,
    
    /// Capability type filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_type: Option<CapabilityType>,
    
    /// Required parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Minimum version requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

/// Query filters for capability discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QueryFilters {
    /// Agent tags to include
    #[serde(default)]
    pub include_tags: Vec<String>,
    
    /// Agent tags to exclude
    #[serde(default)]
    pub exclude_tags: Vec<String>,
    
    /// Maximum cost per request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<f64>,
    
    /// Required agent status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<crate::AgentStatus>,
    
    /// Geographic region preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Capability match result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityMatch {
    /// Matched agent card
    pub agent_card: AgentCard,
    
    /// Match score (0.0 - 1.0)
    pub score: f64,
    
    /// Matched capabilities
    pub matched_capabilities: Vec<String>,
    
    /// Missing required capabilities
    pub missing_capabilities: Vec<String>,
    
    /// Match details
    pub details: MatchDetails,
}

/// Detailed match information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchDetails {
    /// Required capabilities match score
    pub required_score: f64,
    
    /// Optional capabilities match score
    pub optional_score: f64,
    
    /// Filter match score
    pub filter_score: f64,
    
    /// Cost score (lower cost = higher score)
    pub cost_score: f64,
    
    /// Availability score
    pub availability_score: f64,
}

/// Capability discovery service
pub struct CapabilityDiscovery {
    /// Registered agent cards
    agent_cards: HashMap<String, AgentCard>,
}

impl CapabilityDiscovery {
    /// Create a new capability discovery service
    pub fn new() -> Self {
        Self {
            agent_cards: HashMap::new(),
        }
    }
    
    /// Register an agent card
    pub fn register_agent(&mut self, agent_card: AgentCard) {
        self.agent_cards.insert(agent_card.id.clone(), agent_card);
    }
    
    /// Unregister an agent
    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.agent_cards.remove(agent_id);
    }
    
    /// Update an agent card
    pub fn update_agent(&mut self, agent_card: AgentCard) {
        self.agent_cards.insert(agent_card.id.clone(), agent_card);
    }
    
    /// Get an agent card by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentCard> {
        self.agent_cards.get(agent_id)
    }
    
    /// List all registered agents
    pub fn list_agents(&self) -> Vec<&AgentCard> {
        self.agent_cards.values().collect()
    }
    
    /// Discover agents matching capability requirements
    pub fn discover(&self, query: &CapabilityQuery) -> Vec<CapabilityMatch> {
        let mut matches = Vec::new();
        
        for agent_card in self.agent_cards.values() {
            // Skip expired cards
            if agent_card.is_expired() {
                continue;
            }
            
            if let Some(capability_match) = self.match_agent(agent_card, query) {
                matches.push(capability_match);
            }
        }
        
        // Sort by score (highest first)
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Limit results
        matches.truncate(query.max_results);
        
        matches
    }
    
    /// Match a single agent against capability requirements
    fn match_agent(&self, agent_card: &AgentCard, query: &CapabilityQuery) -> Option<CapabilityMatch> {
        let mut matched_capabilities = Vec::new();
        let mut missing_capabilities = Vec::new();
        let mut required_score = 0.0;
        let mut optional_score = 0.0;
        
        // Check required capabilities
        for requirement in &query.required {
            if let Some(capability) = self.find_matching_capability(agent_card, requirement) {
                matched_capabilities.push(capability.name.clone());
                required_score += 1.0;
            } else {
                missing_capabilities.push(requirement.name.clone());
            }
        }
        
        // If any required capabilities are missing, this agent doesn't match
        if !missing_capabilities.is_empty() {
            return None;
        }
        
        // Check optional capabilities
        for requirement in &query.optional {
            if let Some(capability) = self.find_matching_capability(agent_card, requirement) {
                matched_capabilities.push(capability.name.clone());
                optional_score += 1.0;
            }
        }
        
        // Calculate scores
        let required_score = if query.required.is_empty() {
            1.0
        } else {
            required_score / query.required.len() as f64
        };
        
        let optional_score = if query.optional.is_empty() {
            1.0
        } else {
            optional_score / query.optional.len() as f64
        };
        
        let filter_score = self.calculate_filter_score(agent_card, &query.filters);
        let cost_score = self.calculate_cost_score(agent_card, &query.filters);
        let availability_score = self.calculate_availability_score(agent_card);
        
        // Calculate overall score (weighted average)
        let overall_score = (required_score * 0.4) + 
                           (optional_score * 0.2) + 
                           (filter_score * 0.2) + 
                           (cost_score * 0.1) + 
                           (availability_score * 0.1);
        
        Some(CapabilityMatch {
            agent_card: agent_card.clone(),
            score: overall_score,
            matched_capabilities,
            missing_capabilities,
            details: MatchDetails {
                required_score,
                optional_score,
                filter_score,
                cost_score,
                availability_score,
            },
        })
    }
    
    /// Find a capability that matches the requirement
    fn find_matching_capability<'a>(&self, agent_card: &'a AgentCard, requirement: &CapabilityRequirement) -> Option<&'a Capability> {
        agent_card.capabilities.iter().find(|capability| {
            // Check if capability is available
            if !capability.available {
                return false;
            }
            
            // Check name match (exact or pattern)
            if capability.name != requirement.name && !self.matches_pattern(&capability.name, &requirement.name) {
                return false;
            }
            
            // Check type match
            if let Some(required_type) = &requirement.capability_type {
                if &capability.capability_type != required_type {
                    return false;
                }
            }
            
            true
        })
    }
    
    /// Check if a capability name matches a pattern
    fn matches_pattern(&self, name: &str, pattern: &str) -> bool {
        // Simple wildcard matching (* and ?)
        if pattern.contains('*') || pattern.contains('?') {
            // Convert pattern to regex-like matching
            let regex_pattern = pattern
                .replace('*', ".*")
                .replace('?', ".");

            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                return regex.is_match(name);
            }
        }

        false
    }
    
    /// Calculate filter match score
    fn calculate_filter_score(&self, agent_card: &AgentCard, filters: &QueryFilters) -> f64 {
        let mut score = 1.0;
        
        // Check include tags
        if !filters.include_tags.is_empty() {
            let matching_tags = filters.include_tags.iter()
                .filter(|tag| agent_card.tags.contains(tag))
                .count();
            score *= matching_tags as f64 / filters.include_tags.len() as f64;
        }
        
        // Check exclude tags
        for exclude_tag in &filters.exclude_tags {
            if agent_card.tags.contains(exclude_tag) {
                score *= 0.5; // Penalty for having excluded tags
            }
        }
        
        // Check status
        if let Some(required_status) = &filters.status {
            if &agent_card.status != required_status {
                score *= 0.8; // Penalty for wrong status
            }
        }
        
        score.max(0.0).min(1.0)
    }
    
    /// Calculate cost score (lower cost = higher score)
    fn calculate_cost_score(&self, agent_card: &AgentCard, filters: &QueryFilters) -> f64 {
        if let Some(max_cost) = filters.max_cost {
            let avg_cost = agent_card.capabilities.iter()
                .filter_map(|c| c.cost.as_ref())
                .map(|cost| cost.amount)
                .fold(0.0, |acc, cost| acc + cost) / agent_card.capabilities.len() as f64;
            
            if avg_cost > max_cost {
                return 0.0; // Exceeds budget
            }
            
            // Higher score for lower cost
            1.0 - (avg_cost / max_cost).min(1.0)
        } else {
            1.0 // No cost constraint
        }
    }
    
    /// Calculate availability score
    fn calculate_availability_score(&self, agent_card: &AgentCard) -> f64 {
        match agent_card.status {
            crate::AgentStatus::Online => 1.0,
            crate::AgentStatus::Busy => 0.7,
            crate::AgentStatus::Maintenance => 0.3,
            crate::AgentStatus::Offline => 0.0,
            crate::AgentStatus::Unknown => 0.5,
        }
    }
}

impl Default for CapabilityDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

fn default_max_results() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentCard, Capability, CapabilityType, AgentStatus};

    #[test]
    fn test_capability_discovery() {
        let mut discovery = CapabilityDiscovery::new();
        
        // Create test agent
        let agent_card = AgentCard::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            "1.0.0".to_string(),
        ).add_capability(
            Capability::new(
                "text_generation".to_string(),
                "Generate text".to_string(),
                CapabilityType::TextGeneration,
            )
        );
        
        discovery.register_agent(agent_card);
        
        // Create query
        let query = CapabilityQuery {
            required: vec![
                CapabilityRequirement {
                    name: "text_generation".to_string(),
                    capability_type: Some(CapabilityType::TextGeneration),
                    parameters: HashMap::new(),
                    min_version: None,
                }
            ],
            optional: Vec::new(),
            filters: QueryFilters::default(),
            max_results: 10,
        };
        
        // Discover agents
        let matches = discovery.discover(&query);
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].agent_card.id, "agent1");
        assert!(matches[0].score > 0.0);
    }
}
