use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub model_name: String,

    #[arg(long, value_name = "PATH")]
    pub onnx_runtime_path: Option<PathBuf>,

    #[arg(short = 'i', long, required_unless_present = "live_stream")]
    pub input_video_path: Option<String>,

    #[arg(short, long)]
    pub output_video_path: String,

    #[arg(long, short)]
    pub live_stream: bool,
}
