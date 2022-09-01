use plotters::prelude::*;

use std::error::Error;
use plotters::coord::Shift;
use plotters::prelude::full_palette::{GREY, ORANGE, RED_900};

const PRECISION: usize = 10000;

#[derive(Debug)]
struct Settings {
    min_bitrate: u32,
    max_bitrate: u32,
    do_fec_threshold: u32,
    do_fec: bool,
}

#[derive(Debug)]
struct CalcResult {
    fec_percentage: f64,
    fec_bitrate: f64,
    encoders_bitrate: f64,
    total_bitrate: f64,
}

fn calc_old(s: &Settings, bitrate: u32) -> CalcResult {
    let fec_ratio = {
        if s.do_fec && bitrate > s.do_fec_threshold {
            (bitrate as f64 - s.do_fec_threshold as f64)
                / (s.max_bitrate as f64 - s.do_fec_threshold as f64)
        } else {
            0f64
        }
    };

    let fec_percentage = fec_ratio * 50f64;
    let encoders_bitrate = (bitrate as f64) / (1. + (fec_percentage / 100.));

    let fec_bitrate = (fec_percentage / 100.) * bitrate as f64;
    let total_bitrate = encoders_bitrate + fec_bitrate;

    return CalcResult {
        fec_percentage,
        fec_bitrate,
        encoders_bitrate,
        total_bitrate,
    };
}

fn calc_new(s: &Settings, bitrate: u32) -> CalcResult {
    let fec_ratio = {
        if s.do_fec && bitrate > s.do_fec_threshold {
            (bitrate as f64 - s.do_fec_threshold as f64)
                / (s.max_bitrate as f64 - s.do_fec_threshold as f64)
        } else {
            0f64
        }
    };

    let fec_percentage = fec_ratio * 50f64;
    let encoders_bitrate = (bitrate as f64) / (1. + (fec_percentage / 100.));

    let fec_bitrate = (fec_percentage / 100.) * bitrate as f64;
    let total_bitrate = encoders_bitrate + fec_bitrate;

    return CalcResult {
        fec_percentage,
        fec_bitrate,
        encoders_bitrate,
        total_bitrate,
    };
}

fn draw_chart(s: &Settings, root: &DrawingArea<SVGBackend, Shift>, calc: fn(&Settings, u32) -> CalcResult, caption: &str) -> Result<(), Box<dyn Error>> {
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(10)
        .y_label_area_size(75)
        .right_y_label_area_size(50)
        .margin(40)
        .caption(caption, ("sans-serif", 50.0).into_font())
        .build_cartesian_2d(0f32..(s.max_bitrate as f32), 0f32..(s.max_bitrate as f32 * 1.5f32))?
        .set_secondary_coord(0f32..(s.max_bitrate as f32), 0f32..150f32);

    chart
        .configure_mesh()
        .y_desc("Bitrate")
        .y_label_formatter(&|x| format!("{}", x))
        .draw()?;

    chart
        .configure_secondary_axes()
        .y_desc("Percentage (%)")
        .draw()?;


    let x_axis = (s.min_bitrate..s.max_bitrate).step_by(PRECISION);

    chart
        .draw_series(LineSeries::new(
            x_axis.clone().map(|x| (x as f32, s.max_bitrate as f32)),
            &BLACK,
        ))?
        .label("max_bitrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));
    chart
        .draw_series(LineSeries::new(
            x_axis.clone().map(|x| (x as f32, x as f32)),
            &RED_900,
        ))?
        .label("estimated_bitrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED_900));
    chart
        .draw_series(LineSeries::new(
            x_axis.clone().map(|x| {
                let r = calc(&s, x);
                (x as f32, r.encoders_bitrate as f32)
            }),
            &BLUE,
        ))?
        .label("encoders_bitrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    chart
        .draw_series(LineSeries::new(
            x_axis.clone().map(|x| {
                let r = calc(&s, x);
                (x as f32, r.fec_bitrate as f32)
            }),
            &ORANGE,
        ))?
        .label("fec_bitrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &ORANGE));
    chart
        .draw_secondary_series(LineSeries::new(
            x_axis.clone().map(|x| {
                let r = calc(&s, x);
                (x as f32, r.fec_percentage as f32)
            }),
            &GREY,
        ))?
        .label("fec_percentage (%)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREY));

    chart
        .draw_series(LineSeries::new(
            x_axis.clone().map(|x| {
                let r = calc(&s, x);
                (x as f32, r.total_bitrate as f32)
            }),
            &GREEN,
        ))?
        .label("total_bitrate = fec_bitrate + encoders_bitrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .background_style(&RGBColor(128, 128, 128))
        .draw()?;

    Ok(())
}

fn render(s: &Settings) -> Result<(), Box<dyn Error>> {
    let filename = format!("assets/bitrate-calc-{}-{}-{}-{}.svg", s.min_bitrate, s.max_bitrate, s.do_fec, s.do_fec_threshold);
    let root = SVGBackend::new(filename.as_str(), (1920 * 2, 1080)).into_drawing_area();
    root.fill(&WHITE)?;

    // let (left, right) = root.split_horizontally(1920);

    // charts
    draw_chart(s, &root, calc_old, "encoders_bitrate = (bitrate as f64) / (1. + (fec_percentage / 100.))")?;
    // draw_chart(s, &left, calc_old, "encoders_bitrate = (bitrate as f64) / (1. + (fec_percentage / 100.))")?;
    // draw_chart(s, &right, calc_new, "encoders_bitrate = (bitrate as f64) / (1. + (fec_percentage / 100.))")?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let do_fec = true;
    let do_fec_threshold = 2_000_000;
    let min_bitrate = 1_000;

    let settings = (1..=8)
        .map(|mult| mult * 1_024_000)
        .map(|max_bitrate| Settings { min_bitrate, max_bitrate, do_fec_threshold, do_fec });

    for s in settings {
        render(&s)?
    }
    Ok(())
}
