use crate::codecs;

pub fn map_video_codec(mkv_codec_id: &str) -> &'static str {
	match mkv_codec_id {
		"V_MPEG4/ISO/AVC" | "V_MPEG4/ISO/AVC1" => codecs::video::H264,
		"V_MPEG4/ISO/HEVC" | "V_MPEGH/ISO/HEVC" => codecs::video::H265,
		"V_VP8" => codecs::video::VP8,
		"V_VP9" => codecs::video::VP9,
		"V_VP10" => codecs::video::VP10,
		"V_AV1" => codecs::video::AV1,
		"V_MPEG1" => codecs::video::MPEG1,
		"V_MPEG2" => codecs::video::MPEG2,
		"V_MPEG4/ISO" | "V_MPEG4/ISO/SP" | "V_MPEG4/ISO/ASP" => codecs::video::MPEG4,
		"V_THEORA" => codecs::video::THEORA,
		"V_VP6" | "V_VP6F" | "V_VP6A" => codecs::video::VP6,
		"V_PRORES" => codecs::video::PRORES,
		"V_DNXHD" => codecs::video::DNXHD,
		"V_DNXHR" => codecs::video::DNXHR,
		"V_MJPEG" => codecs::video::MJPEG,
		"V_JPEG2000" => codecs::video::JPEG2000,
		"V_UNCOMPRESSED" => codecs::video::RAWVIDEO,
		_ => codecs::UNKNOWN,
	}
}

pub fn map_audio_codec(mkv_codec_id: &str) -> &'static str {
	match mkv_codec_id {
		"A_AAC" | "A_AAC/MPEG2/LC" | "A_AAC/MPEG4/LC" | "A_AAC/MPEG4/LC/SBR" => codecs::audio::AAC,
		"A_MPEG/L1" => codecs::audio::MP1,
		"A_MPEG/L2" => codecs::audio::MP2,
		"A_MPEG/L3" => codecs::audio::MP3,
		"A_OPUS" => codecs::audio::OPUS,
		"A_VORBIS" => codecs::audio::VORBIS,
		"A_AC3" => codecs::audio::AC3,
		"A_EAC3" => codecs::audio::EAC3,
		"A_FLAC" => codecs::audio::FLAC,
		"A_ALAC" => codecs::audio::ALAC,
		"A_WAVPACK" => codecs::audio::WAVPACK,
		"A_TTA1" => codecs::audio::TTA,
		"A_WMA1" | "A_WMA2" | "A_WMA3" => codecs::audio::WMA,
		"A_ATRAC3" | "A_ATRAC3PLUS" => codecs::audio::ATRAC3,
		"A_AMR/NB" => codecs::audio::AMR_NB,
		"A_AMR/WB" => codecs::audio::AMR_WB,
		"A_PCM/INT/LIT" | "A_PCM/INT/BIG" => codecs::audio::PCM_S16LE,
		"A_PCM/FLOAT/IEEE" => codecs::audio::PCM_F32LE,
		"A_MUSEPACK7" | "A_MUSEPACK8" => codecs::audio::MUSEPACK,
		"A_DSD_LSBF" => codecs::audio::DSD_LSBF,
		"A_DSD_MSBF" => codecs::audio::DSD_MSBF,
		"A_DSD_LSBF_PLANAR" => codecs::audio::DSD_LSBF_PLANAR,
		"A_DSD_MSBF_PLANAR" => codecs::audio::DSD_MSBF_PLANAR,
		"A_REAL/14_4" | "A_REAL/28_8" | "A_REAL/COOK" | "A_REAL/SIPR" => codecs::audio::REALAUDIOL,
		_ => codecs::UNKNOWN,
	}
}

pub fn map_subtitle_codec(mkv_codec_id: &str) -> &'static str {
	match mkv_codec_id {
		"S_TEXT/UTF8" | "S_TEXT/ASCII" | "S_TEXT/USF" => codecs::subtitle::SRT,
		"S_TEXT/SSA" => codecs::subtitle::SSA,
		"S_TEXT/ASS" => codecs::subtitle::ASS,
		"S_TEXT/WEBVTT" => codecs::subtitle::VTT,
		"S_TEXT/MOV" => codecs::subtitle::MOV_TEXT,
		"S_VOBSUB" => codecs::subtitle::DVDSUB,
		"S_HDMV/PGS" => codecs::subtitle::HDMV_PGS,
		"S_DVBSUB" => codecs::subtitle::DVB_SUBTITLE,
		"S_IMAGE/BMP" => codecs::subtitle::BMP,
		_ => codecs::UNKNOWN,
	}
}
