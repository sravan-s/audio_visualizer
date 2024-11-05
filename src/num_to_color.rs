pub fn number_to_color(value: Option<&f32>) -> egui::Color32 {
    if value.is_none() {
        return egui::Color32::from_rgb(0, 0, 0);
    }

    let value = value.unwrap();
    // Clamp the value between -1.0 and 1.0
    let clamped = value.clamp(-1.0, 1.0);

    // Map -1 to 1 range to 0 to 255 for color intensity
    let red = ((1.0 - clamped) * 127.5).round() as u8;
    let blue = ((clamped + 1.0) * 127.5).round() as u8;

    // Interpolates white to red and blue
    egui::Color32::from_rgb(red, 255 - red.max(blue), blue)
}
