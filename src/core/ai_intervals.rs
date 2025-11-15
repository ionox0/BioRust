/// AI Interval System for Performance Optimization
/// Converts frame-based AI updates to interval-based updates to dramatically reduce CPU usage

use bevy::prelude::*;
use std::time::Duration;

/// Resource to manage AI update intervals
#[derive(Resource, Debug)]
pub struct AIIntervals {
    /// Intelligence and scouting updates - every 1 second
    pub intelligence_timer: Timer,
    /// Strategic decision making - every 2 seconds  
    pub strategy_timer: Timer,
    /// Tactical decisions - every 0.5 seconds
    pub tactics_timer: Timer,
    /// Economy optimization - every 1.5 seconds
    pub economy_timer: Timer,
    /// Unit management - every 0.3 seconds
    pub unit_management_timer: Timer,
    /// Combat decisions - every 0.2 seconds (more responsive)
    pub combat_timer: Timer,
    /// Resource management - every 1 second
    pub resource_timer: Timer,
}

impl Default for AIIntervals {
    fn default() -> Self {
        Self {
            intelligence_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            strategy_timer: Timer::from_seconds(2.0, TimerMode::Repeating), 
            tactics_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            economy_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            unit_management_timer: Timer::from_seconds(0.3, TimerMode::Repeating),
            combat_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            resource_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

impl AIIntervals {
    /// Check if intelligence systems should update this frame
    pub fn should_update_intelligence(&self) -> bool {
        self.intelligence_timer.just_finished()
    }

    /// Check if strategy systems should update this frame
    pub fn should_update_strategy(&self) -> bool {
        self.strategy_timer.just_finished()
    }

    /// Check if tactical systems should update this frame  
    pub fn should_update_tactics(&self) -> bool {
        self.tactics_timer.just_finished()
    }

    /// Check if economy systems should update this frame
    pub fn should_update_economy(&self) -> bool {
        self.economy_timer.just_finished()
    }

    /// Check if unit management should update this frame
    pub fn should_update_unit_management(&self) -> bool {
        self.unit_management_timer.just_finished()
    }

    /// Check if combat systems should update this frame
    pub fn should_update_combat(&self) -> bool {
        self.combat_timer.just_finished()
    }

    /// Check if resource management should update this frame
    pub fn should_update_resources(&self) -> bool {
        self.resource_timer.just_finished()
    }

    /// Adjust intervals based on game state (e.g., more frequent updates during combat)
    pub fn set_combat_mode(&mut self, in_combat: bool) {
        if in_combat {
            // More frequent updates during active combat
            self.combat_timer.set_duration(Duration::from_millis(100)); // 10fps
            self.tactics_timer.set_duration(Duration::from_millis(250)); // 4fps
            self.unit_management_timer.set_duration(Duration::from_millis(150)); // ~7fps
        } else {
            // Normal peacetime intervals
            self.combat_timer.set_duration(Duration::from_millis(200)); // 5fps
            self.tactics_timer.set_duration(Duration::from_millis(500)); // 2fps
            self.unit_management_timer.set_duration(Duration::from_millis(300)); // ~3fps
        }
    }

    /// Adjust intervals based on unit count (fewer units = less frequent updates)
    pub fn adjust_for_unit_count(&mut self, unit_count: usize) {
        let scale_factor = match unit_count {
            0..=20 => 1.5,      // Fewer units = longer intervals
            21..=50 => 1.0,     // Normal intervals  
            51..=100 => 0.8,    // More units = slightly shorter intervals
            _ => 0.6,           // Many units = shorter intervals for responsiveness
        };

        self.intelligence_timer.set_duration(Duration::from_secs_f32(1.0 * scale_factor));
        self.strategy_timer.set_duration(Duration::from_secs_f32(2.0 * scale_factor));
        self.economy_timer.set_duration(Duration::from_secs_f32(1.5 * scale_factor));
        self.resource_timer.set_duration(Duration::from_secs_f32(1.0 * scale_factor));
    }
}

/// System to tick all AI interval timers
pub fn ai_interval_tick_system(
    mut intervals: ResMut<AIIntervals>,
    time: Res<Time>,
) {
    intervals.intelligence_timer.tick(time.delta());
    intervals.strategy_timer.tick(time.delta());
    intervals.tactics_timer.tick(time.delta());
    intervals.economy_timer.tick(time.delta());
    intervals.unit_management_timer.tick(time.delta());
    intervals.combat_timer.tick(time.delta());
    intervals.resource_timer.tick(time.delta());
}

/// System to dynamically adjust AI intervals based on game state
pub fn ai_interval_adjustment_system(
    mut intervals: ResMut<AIIntervals>,
    basic_query: Query<&crate::core::components::RTSUnit>,
    combat_query: Query<&crate::core::components::Combat>,
) {
    // Count total units to adjust intervals
    let unit_count = basic_query.iter().count();
    
    // Check if any AI units are in combat
    let in_combat = combat_query
        .iter()
        .any(|combat| combat.auto_attack && combat.target.is_some());

    // Adjust intervals every few seconds (not every frame)
    if intervals.strategy_timer.just_finished() {
        intervals.adjust_for_unit_count(unit_count);
        intervals.set_combat_mode(in_combat);
        
        if unit_count > 0 && intervals.strategy_timer.duration().as_secs() > 0 {
            debug!("AI Intervals adjusted: {} units, combat: {}", unit_count, in_combat);
        }
    }
}

/// Condition function for intelligence systems
pub fn should_run_intelligence(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_intelligence()
}

/// Condition function for strategy systems  
pub fn should_run_strategy(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_strategy()
}

/// Condition function for tactical systems
pub fn should_run_tactics(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_tactics()
}

/// Condition function for economy systems
pub fn should_run_economy(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_economy()
}

/// Condition function for unit management systems
pub fn should_run_unit_management(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_unit_management()
}

/// Condition function for combat systems
pub fn should_run_combat(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_combat()
}

/// Condition function for resource systems
pub fn should_run_resources(intervals: Res<AIIntervals>) -> bool {
    intervals.should_update_resources()
}

/// Plugin to add AI interval management
pub struct AIIntervalsPlugin;

impl Plugin for AIIntervalsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AIIntervals>()
            .add_systems(
                Update, 
                (
                    ai_interval_tick_system,
                    ai_interval_adjustment_system.after(ai_interval_tick_system),
                )
            );
    }
}