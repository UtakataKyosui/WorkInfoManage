// Physics-based animation for menu selection
use crate::app::{App, CurrentScreen};

pub fn update_physics(app: &mut App) {
    if app.current_screen == CurrentScreen::Menu {
        let stiffness = 0.3;
        let damping = 0.6;
        let target = app.menu_selection as f32;
        let diff = target - app.animation.visual_selection;
        
        let acceleration = diff * stiffness;
        app.animation.visual_velocity += acceleration;
        app.animation.visual_velocity *= damping;
        
        app.animation.visual_selection += app.animation.visual_velocity;
        
        if app.animation.visual_velocity.abs() < 0.01 && diff.abs() < 0.01 {
            app.animation.visual_selection = target;
            app.animation.visual_velocity = 0.0;
        }
    }
}
