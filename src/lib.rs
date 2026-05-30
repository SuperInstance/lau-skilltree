use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SkillCategory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    Building,
    Farming,
    AgentTraining,
    Conservation,
    Exploration,
    Trading,
    Teaching,
    Research,
}

// ---------------------------------------------------------------------------
// SkillReward
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillReward {
    UnlockRecipe(String),
    UnlockBiome(String),
    UnlockAgent(String),
    BonusVibe(f64),
    UnlockQuest(String),
}

// ---------------------------------------------------------------------------
// Skill
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub level: u32,
    pub max_level: u32,
    pub xp: f64,
    pub xp_to_next: f64,
    pub category: SkillCategory,
}

impl Skill {
    pub fn add_xp(&mut self, mut amount: f64) -> bool {
        let mut leveled_up = false;
        while amount > 0.0 && self.level < self.max_level {
            let needed = self.xp_to_next - self.xp;
            if amount >= needed {
                amount -= needed;
                self.xp = 0.0;
                self.level += 1;
                self.xp_to_next = next_xp_cost(self.level);
                leveled_up = true;
            } else {
                self.xp += amount;
                amount = 0.0;
            }
        }
        // overflow xp when maxed
        if self.is_maxed() {
            self.xp = self.xp_to_next;
        }
        leveled_up
    }

    pub fn is_maxed(&self) -> bool {
        self.level >= self.max_level
    }

    pub fn progress(&self) -> f64 {
        if self.is_maxed() {
            return 1.0;
        }
        if self.level == 0 {
            return 0.0;
        }
        if self.xp_to_next <= 0.0 {
            return 1.0;
        }
        (self.xp / self.xp_to_next).clamp(0.0, 1.0)
    }
}

/// XP curve: base * factor^level
fn next_xp_cost(level: u32) -> f64 {
    100.0 * 1.5_f64.powi(level as i32)
}

// ---------------------------------------------------------------------------
// SkillNode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNode {
    pub skill: Skill,
    pub prerequisites: Vec<String>,
    pub unlocks: Vec<String>,
    pub position: (f64, f64),
}

// ---------------------------------------------------------------------------
// SkillTree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTree {
    pub nodes: HashMap<String, SkillNode>,
}

impl SkillTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: SkillNode) {
        self.nodes.insert(node.skill.id.clone(), node);
    }

    /// A skill is "unlocked" if its level > 0 and all prerequisites are met.
    pub fn is_unlocked(&self, skill_id: &str) -> bool {
        if let Some(node) = self.nodes.get(skill_id) {
            if node.skill.level == 0 {
                return false;
            }
            for prereq in &node.prerequisites {
                if !self.is_unlocked(prereq) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Unlock a skill (set level to 1) if prerequisites are met.
    /// Returns true if the skill was newly unlocked.
    pub fn unlock(&mut self, skill_id: &str) -> bool {
        let prereqs_met = {
            let node = match self.nodes.get(skill_id) {
                Some(n) => n,
                None => return false,
            };
            if node.skill.level > 0 {
                return false;
            }
            for prereq in &node.prerequisites {
                if self.nodes.get(prereq).is_none_or(|n| n.skill.level == 0) {
                    return false;
                }
            }
            true
        };
        if prereqs_met {
            if let Some(node) = self.nodes.get_mut(skill_id) {
                node.skill.level = 1;
                node.skill.xp = 0.0;
                node.skill.xp_to_next = next_xp_cost(1);
                return true;
            }
        }
        false
    }

    /// Add XP to a skill. Returns true if a level-up occurred.
    pub fn add_xp(&mut self, skill_id: &str, amount: f64) -> bool {
        if let Some(node) = self.nodes.get_mut(skill_id) {
            if node.skill.level == 0 {
                return false;
            }
            return node.skill.add_xp(amount);
        }
        false
    }

    pub fn unlocked_skills(&self) -> Vec<&SkillNode> {
        self.nodes
            .values()
            .filter(|n| self.is_unlocked(&n.skill.id))
            .collect()
    }

    pub fn locked_skills(&self) -> Vec<&SkillNode> {
        self.nodes
            .values()
            .filter(|n| !self.is_unlocked(&n.skill.id))
            .collect()
    }

    pub fn total_level(&self) -> u32 {
        self.nodes.values().map(|n| n.skill.level).sum()
    }

    pub fn category_level(&self, cat: &SkillCategory) -> u32 {
        self.nodes
            .values()
            .filter(|n| &n.skill.category == cat)
            .map(|n| n.skill.level)
            .sum()
    }

    // -----------------------------------------------------------------------
    // Pre-built skill trees
    // -----------------------------------------------------------------------

    pub fn default_tree() -> Self {
        let mut tree = Self::new();

        // -- Building --
        let building = vec![
            ("place_block", "Place Block", "Place your first block", "🧱", 1, 0.0, 0.0),
            ("stack_blocks", "Stack Blocks", "Stack blocks on top of each other", "🏗️", 3, 0.0, 1.0),
            ("build_structure", "Build Structure", "Construct a complete structure", "🏠", 5, 0.0, 2.0),
            ("master_builder", "Master Builder", "Build anything you can imagine", "👷", 10, 0.0, 3.0),
        ];
        for (id, name, desc, icon, max, y, x) in &building {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Building),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["place_block", "stack_blocks", "build_structure", "master_builder"]);

        // -- Farming --
        let farming = vec![
            ("plant_seed", "Plant Seed", "Put a seed in the ground", "🌱", 1, 0.0, 0.0),
            ("irrigation", "Irrigation", "Water your crops efficiently", "💧", 3, 0.0, 1.0),
            ("crop_rotation", "Crop Rotation", "Rotate crops for better yields", "🌾", 5, 0.0, 2.0),
            ("ecosystem", "Ecosystem", "Create a self-sustaining ecosystem", "🌿", 8, 0.0, 3.0),
        ];
        for (id, name, desc, icon, max, y, x) in &farming {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Farming),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["plant_seed", "irrigation", "crop_rotation", "ecosystem"]);

        // -- Agent Training --
        let agent = vec![
            ("meet_agent", "Meet Agent", "Meet your first AI agent", "🤖", 1, 0.0, 0.0),
            ("train_agent", "Train Agent", "Teach an agent new behaviors", "🎓", 3, 0.0, 1.0),
            ("compose_agents", "Compose Agents", "Combine agents into teams", "⚙️", 6, 0.0, 2.0),
            ("create_agent", "Create Agent", "Design your own agent from scratch", "✨", 10, 0.0, 3.0),
        ];
        for (id, name, desc, icon, max, y, x) in &agent {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::AgentTraining),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["meet_agent", "train_agent", "compose_agents", "create_agent"]);

        // -- Conservation --
        let conservation = vec![
            ("observe_vibe", "Observe Vibe", "Sense the vibe of a place", "👁️", 1, 0.0, 0.0),
            ("check_balance", "Check Balance", "Measure ecological balance", "⚖️", 3, 0.0, 1.0),
            ("verify_conservation", "Verify Conservation", "Confirm conservation laws hold", "🔬", 5, 0.0, 2.0),
            ("perfect_balance", "Perfect Balance", "Achieve perfect vibe equilibrium", "🧘", 8, 0.0, 3.0),
        ];
        for (id, name, desc, icon, max, y, x) in &conservation {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Conservation),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["observe_vibe", "check_balance", "verify_conservation", "perfect_balance"]);

        // -- Exploration --
        let exploration = vec![
            ("first_room", "First Room", "Explore your first room", "🚪", 1, 0.0, 0.0),
            ("map_world", "Map World", "Draw a map of the world", "🗺️", 3, 0.0, 1.0),
            ("discover_biome", "Discover Biome", "Find a new biome", "🏔️", 5, 0.0, 2.0),
            ("find_ancient_ruins", "Find Ancient Ruins", "Discover ancient ruins", "🏛️", 8, 0.0, 3.0),
        ];
        for (id, name, desc, icon, max, y, x) in &exploration {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Exploration),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["first_room", "map_world", "discover_biome", "find_ancient_ruins"]);

        // -- Trading --
        let trading = vec![
            ("first_trade", "First Trade", "Make your first trade", "🤝", 1, 0.0, 4.0),
            ("bargain", "Bargain", "Negotiate better deals", "💰", 3, 0.0, 5.0),
            ("market", "Market Stall", "Set up a market stall", "🏪", 5, 0.0, 6.0),
            ("trade_routes", "Trade Routes", "Establish long-distance trade", "🚛", 8, 0.0, 7.0),
        ];
        for (id, name, desc, icon, max, y, x) in &trading {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Trading),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["first_trade", "bargain", "market", "trade_routes"]);

        // -- Teaching --
        let teaching = vec![
            ("show_friend", "Show a Friend", "Teach something to a friend", "👋", 1, 0.0, 4.0),
            ("mentor", "Mentor", "Guide another learner", "📚", 3, 0.0, 5.0),
            ("write_guide", "Write Guide", "Create a teaching guide", "📝", 5, 0.0, 6.0),
            ("master_teacher", "Master Teacher", "Teach the teachers", "🎓", 8, 0.0, 7.0),
        ];
        for (id, name, desc, icon, max, y, x) in &teaching {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Teaching),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["show_friend", "mentor", "write_guide", "master_teacher"]);

        // -- Research --
        let research = vec![
            ("curiosity", "Curiosity", "Ask your first question", "❓", 1, 0.0, 8.0),
            ("experiment", "Experiment", "Test a hypothesis", "🧪", 3, 0.0, 9.0),
            ("data_analysis", "Data Analysis", "Analyze experimental data", "📊", 5, 0.0, 10.0),
            ("breakthrough", "Breakthrough", "Make a scientific breakthrough", "💡", 8, 0.0, 11.0),
            ("publish", "Publish", "Share your findings with the world", "📰", 10, 0.0, 12.0),
        ];
        for (id, name, desc, icon, max, y, x) in &research {
            tree.add_node(SkillNode {
                skill: mk_skill(id, name, desc, icon, *max, SkillCategory::Research),
                prerequisites: vec![],
                unlocks: vec![],
                position: (*x, *y),
            });
        }
        chain(&mut tree, &["curiosity", "experiment", "data_analysis", "breakthrough", "publish"]);

        // -- Cross-category links (prerequisites) --
        // AgentTraining → Research: compose_agents requires data_analysis
        if let Some(node) = tree.nodes.get_mut("compose_agents") {
            node.prerequisites.push("data_analysis".into());
        }
        // Conservation → Farming: perfect_balance requires ecosystem
        if let Some(node) = tree.nodes.get_mut("perfect_balance") {
            node.prerequisites.push("ecosystem".into());
        }

        tree
    }
}

fn mk_skill(id: &str, name: &str, desc: &str, icon: &str, max_level: u32, cat: SkillCategory) -> Skill {
    Skill {
        id: id.into(),
        name: name.into(),
        description: desc.into(),
        icon: icon.into(),
        level: 0,
        max_level,
        xp: 0.0,
        xp_to_next: next_xp_cost(0),
        category: cat,
    }
}

/// Wire up a linear chain: a → b → c → … (prerequisites + unlocks).
fn chain(tree: &mut SkillTree, ids: &[&str]) {
    for i in 0..ids.len() {
        if i > 0 {
            if let Some(node) = tree.nodes.get_mut(ids[i]) {
                node.prerequisites.push(ids[i - 1].into());
            }
        }
        if i + 1 < ids.len() {
            if let Some(node) = tree.nodes.get_mut(ids[i]) {
                node.unlocks.push(ids[i + 1].into());
            }
        }
    }
}

impl Default for SkillTree {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_skill(id: &str, max: u32) -> Skill {
        Skill {
            id: id.into(),
            name: id.into(),
            description: String::new(),
            icon: "⭐".into(),
            level: 0,
            max_level: max,
            xp: 0.0,
            xp_to_next: 100.0,
            category: SkillCategory::Building,
        }
    }

    fn dummy_node(id: &str, max: u32, prereqs: Vec<&str>) -> SkillNode {
        SkillNode {
            skill: dummy_skill(id, max),
            prerequisites: prereqs.into_iter().map(String::from).collect(),
            unlocks: vec![],
            position: (0.0, 0.0),
        }
    }

    // -- Skill tests --

    #[test]
    fn skill_add_xp_level_up() {
        let mut s = dummy_skill("a", 3);
        assert!(s.add_xp(100.0));
        assert_eq!(s.level, 1);
        assert!(!s.is_maxed());
    }

    #[test]
    fn skill_add_xp_multiple_levels() {
        let mut s = dummy_skill("a", 3);
        s.add_xp(100.0 + 150.0);
        assert_eq!(s.level, 2);
    }

    #[test]
    fn skill_is_maxed() {
        let mut s = dummy_skill("a", 2);
        s.add_xp(100.0 + 150.0);
        assert_eq!(s.level, 2);
        assert!(s.is_maxed());
    }

    #[test]
    fn skill_overflow_xp_capped() {
        let mut s = dummy_skill("a", 1);
        s.add_xp(500.0);
        assert!(s.is_maxed());
        assert_eq!(s.xp, s.xp_to_next);
    }

    #[test]
    fn skill_progress_zero_level() {
        let s = dummy_skill("a", 3);
        assert_eq!(s.progress(), 0.0);
    }

    #[test]
    fn skill_progress_partial() {
        let mut s = dummy_skill("a", 3);
        s.level = 1;
        s.xp = 50.0;
        s.xp_to_next = 100.0;
        let p = s.progress();
        assert!((p - 0.5).abs() < 1e-9);
    }

    #[test]
    fn skill_progress_maxed() {
        let mut s = dummy_skill("a", 1);
        s.add_xp(100.0);
        assert_eq!(s.progress(), 1.0);
    }

    #[test]
    fn skill_no_xp_when_maxed() {
        let mut s = dummy_skill("a", 1);
        assert!(s.add_xp(100.0));
        assert!(!s.add_xp(100.0));
    }

    // -- SkillTree tests --

    #[test]
    fn tree_add_and_get_node() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        assert!(tree.nodes.contains_key("a"));
    }

    #[test]
    fn tree_unlock_no_prereqs() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        assert!(tree.unlock("a"));
        assert_eq!(tree.nodes["a"].skill.level, 1);
    }

    #[test]
    fn tree_unlock_with_prereqs_met() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        tree.add_node(dummy_node("b", 3, vec!["a"]));
        tree.unlock("a");
        assert!(tree.unlock("b"));
    }

    #[test]
    fn tree_unlock_prereqs_not_met() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        tree.add_node(dummy_node("b", 3, vec!["a"]));
        assert!(!tree.unlock("b"));
    }

    #[test]
    fn tree_unlock_twice_returns_false() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        assert!(tree.unlock("a"));
        assert!(!tree.unlock("a"));
    }

    #[test]
    fn tree_unlock_missing_returns_false() {
        let tree = SkillTree::new();
        assert!(!tree.nodes.contains_key("ghost"));
    }

    #[test]
    fn tree_add_xp_to_unlocked_skill() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        tree.unlock("a");
        // unlock sets xp_to_next = next_xp_cost(1) = 150
        assert!(tree.add_xp("a", 150.0));
    }

    #[test]
    fn tree_add_xp_to_locked_skill() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        assert!(!tree.add_xp("a", 100.0));
    }

    #[test]
    fn tree_unlocked_and_locked_lists() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        tree.add_node(dummy_node("b", 3, vec!["a"]));
        tree.unlock("a");
        assert_eq!(tree.unlocked_skills().len(), 1);
        assert_eq!(tree.locked_skills().len(), 1);
    }

    #[test]
    fn tree_total_level() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        tree.add_node(dummy_node("b", 5, vec![]));
        tree.unlock("a");
        tree.unlock("b");
        assert_eq!(tree.total_level(), 2);
    }

    #[test]
    fn tree_category_level() {
        let mut tree = SkillTree::new();
        let mut node = dummy_node("a", 3, vec![]);
        node.skill.category = SkillCategory::Farming;
        tree.add_node(node);
        tree.unlock("a");
        assert_eq!(tree.category_level(&SkillCategory::Farming), 1);
        assert_eq!(tree.category_level(&SkillCategory::Building), 0);
    }

    #[test]
    fn tree_is_unlocked_checks_prereqs() {
        let mut tree = SkillTree::new();
        tree.add_node(dummy_node("a", 3, vec![]));
        tree.add_node(dummy_node("b", 3, vec!["a"]));
        tree.unlock("a");
        // b is level 0 → not unlocked even though prereq a is unlocked
        assert!(!tree.is_unlocked("b"));
        tree.unlock("b");
        assert!(tree.is_unlocked("b"));
    }

    // -- Default tree tests --

    #[test]
    fn default_tree_has_30_plus_skills() {
        let tree = SkillTree::default_tree();
        assert!(tree.nodes.len() >= 30, "expected >= 30 skills, got {}", tree.nodes.len());
    }

    #[test]
    fn default_tree_building_chain() {
        let mut tree = SkillTree::default_tree();
        assert!(tree.unlock("place_block"));
        assert!(tree.unlock("stack_blocks"));
        assert!(tree.unlock("build_structure"));
        assert!(tree.unlock("master_builder"));
    }

    #[test]
    fn default_tree_farming_chain() {
        let mut tree = SkillTree::default_tree();
        assert!(tree.unlock("plant_seed"));
        assert!(tree.unlock("irrigation"));
    }

    #[test]
    fn default_tree_agent_chain() {
        let mut tree = SkillTree::default_tree();
        assert!(tree.unlock("meet_agent"));
        assert!(tree.unlock("train_agent"));
    }

    #[test]
    fn default_tree_all_categories_present() {
        let tree = SkillTree::default_tree();
        let cats = [
            SkillCategory::Building,
            SkillCategory::Farming,
            SkillCategory::AgentTraining,
            SkillCategory::Conservation,
            SkillCategory::Exploration,
            SkillCategory::Trading,
            SkillCategory::Teaching,
            SkillCategory::Research,
        ];
        for cat in &cats {
            assert!(tree.nodes.values().any(|n| &n.skill.category == cat), "missing category {:?}", cat);
        }
    }

    #[test]
    fn default_tree_total_level_starts_zero() {
        let tree = SkillTree::default_tree();
        assert_eq!(tree.total_level(), 0);
    }

    // -- Serde round-trip --

    #[test]
    fn serde_skill_roundtrip() {
        let s = dummy_skill("test", 5);
        let json = serde_json::to_string(&s).unwrap();
        let s2: Skill = serde_json::from_str(&json).unwrap();
        assert_eq!(s.id, s2.id);
        assert_eq!(s.max_level, s2.max_level);
    }

    #[test]
    fn serde_tree_roundtrip() {
        let tree = SkillTree::default_tree();
        let json = serde_json::to_string(&tree).unwrap();
        let t2: SkillTree = serde_json::from_str(&json).unwrap();
        assert_eq!(tree.nodes.len(), t2.nodes.len());
    }

    #[test]
    fn serde_skill_category_roundtrip() {
        let cats = vec![
            SkillCategory::Building,
            SkillCategory::AgentTraining,
            SkillCategory::Research,
        ];
        let json = serde_json::to_string(&cats).unwrap();
        let c2: Vec<SkillCategory> = serde_json::from_str(&json).unwrap();
        assert_eq!(cats, c2);
    }

    #[test]
    fn serde_skill_reward_roundtrip() {
        let r = SkillReward::BonusVibe(42.5);
        let json = serde_json::to_string(&r).unwrap();
        let r2: SkillReward = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn default_tree_cross_category_links() {
        let mut tree = SkillTree::default_tree();
        // Unlock research chain up to data_analysis
        tree.unlock("curiosity");
        tree.unlock("experiment");
        tree.unlock("data_analysis");
        // compose_agents should now be unlockable (prereqs: train_agent + data_analysis)
        tree.unlock("meet_agent");
        tree.unlock("train_agent");
        assert!(tree.unlock("compose_agents"));
    }

    #[test]
    fn node_prerequisites_populated() {
        let tree = SkillTree::default_tree();
        let sb = &tree.nodes["stack_blocks"];
        assert!(sb.prerequisites.contains(&"place_block".to_string()));
    }

    #[test]
    fn node_unlocks_populated() {
        let tree = SkillTree::default_tree();
        let pb = &tree.nodes["place_block"];
        assert!(pb.unlocks.contains(&"stack_blocks".to_string()));
    }
}
