use winit::event::WindowEvent;

pub fn deref_event<'a>(reference: &WindowEvent) -> WindowEvent<'a> {
    match reference {
        WindowEvent::AxisMotion {
            device_id,
            axis,
            value,
        } => WindowEvent::AxisMotion {
            device_id: *device_id,
            axis: *axis,
            value: *value,
        },
        WindowEvent::CloseRequested => WindowEvent::CloseRequested,
        WindowEvent::CursorEntered { device_id } => WindowEvent::CursorEntered {
            device_id: *device_id,
        },
        WindowEvent::CursorLeft { device_id } => WindowEvent::CursorLeft {
            device_id: *device_id,
        },
        WindowEvent::CursorMoved {
            device_id,
            position,
            modifiers,
        } => WindowEvent::CursorMoved {
            device_id: *device_id,
            position: *position,
            modifiers: *modifiers,
        },
        WindowEvent::Destroyed => WindowEvent::Destroyed,
        WindowEvent::DroppedFile(f) => WindowEvent::DroppedFile(f.clone()),
        WindowEvent::Focused(b) => WindowEvent::Focused(*b),
        WindowEvent::HoveredFile(f) => WindowEvent::HoveredFile(f.clone()),
        WindowEvent::HoveredFileCancelled => WindowEvent::HoveredFileCancelled,
        WindowEvent::Ime(i) => WindowEvent::Ime(i.clone()),
        WindowEvent::KeyboardInput {
            device_id,
            input,
            is_synthetic,
        } => WindowEvent::KeyboardInput {
            device_id: *device_id,
            input: *input,
            is_synthetic: *is_synthetic,
        },
        WindowEvent::ModifiersChanged(m) => WindowEvent::ModifiersChanged(*m),
        WindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
        } => WindowEvent::MouseInput {
            device_id: *device_id,
            state: *state,
            button: *button,
            modifiers: *modifiers,
        },
        WindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
            #[allow(deprecated)]
            modifiers,
        } => WindowEvent::MouseWheel {
            device_id: *device_id,
            delta: *delta,
            phase: *phase,
            modifiers: *modifiers,
        },
        WindowEvent::Moved(p) => WindowEvent::Moved(*p),
        WindowEvent::Occluded(b) => WindowEvent::Occluded(*b),
        WindowEvent::ReceivedCharacter(c) => WindowEvent::ReceivedCharacter(*c),
        WindowEvent::Resized(s) => WindowEvent::Resized(*s),
        WindowEvent::ScaleFactorChanged {
            scale_factor: _,
            new_inner_size: _,
        } => {
            println!("TODO!: find an alternative to the atrocities commited in tar_core/src/convert_event.rs");
            return WindowEvent::Focused(true);
        }
        WindowEvent::SmartMagnify { device_id } => WindowEvent::SmartMagnify {
            device_id: *device_id,
        },
        WindowEvent::ThemeChanged(t) => WindowEvent::ThemeChanged(*t),
        WindowEvent::Touch(t) => WindowEvent::Touch(*t),
        WindowEvent::TouchpadMagnify {
            device_id,
            delta,
            phase,
        } => WindowEvent::TouchpadMagnify {
            device_id: *device_id,
            delta: *delta,
            phase: *phase,
        },
        WindowEvent::TouchpadPressure {
            device_id,
            pressure,
            stage,
        } => WindowEvent::TouchpadPressure {
            device_id: *device_id,
            pressure: *pressure,
            stage: *stage,
        },
        WindowEvent::TouchpadRotate {
            device_id,
            delta,
            phase,
        } => WindowEvent::TouchpadRotate {
            device_id: *device_id,
            delta: *delta,
            phase: *phase,
        },
    }
}
