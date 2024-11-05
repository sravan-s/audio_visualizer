pub fn number_to_circle(value: Option<&f32>) -> (f32, egui::Color32) {
    if value.is_none() {
        return (0.0, egui::Color32::from_rgb(0, 0, 0));
    }

    let value = value.unwrap();
    // Clamp the value between -1.0 and 1.0
    let clamped = value.clamp(-1.0, 1.0);

    // Extract last two decimal digits of the absolute value
    let red = ((1.0 - clamped) * 127.5).round() as u8;
    let blue = ((clamped + 1.0) * 127.5).round() as u8;
    let green = red.abs_diff(blue);
    // Map the value to 0.0 - 200.0
    let radius: f32 = (clamped + 1.0) * 100.0;

    // Interpolates white to red and blue
    (radius, egui::Color32::from_rgb(red, green, blue))
}
