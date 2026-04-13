use crate::interpreter::Interpreter;
use crate::interpreter::intent::Intent;

use super::{cap_opt, cap_str};

/// Parse creative + knowledge app intents: Shruti, Tazama, Rasa, Mneme, Synapse
pub(super) fn parse_creative(interp: &Interpreter, input_lower: &str) -> Option<Intent> {
    // --- Shruti DAW intents ---
    if let Some(caps) = interp.try_captures("shruti_session", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::ShrutiSession { action, name });
        }
    }

    if let Some(caps) = interp.try_captures("shruti_track", input_lower) {
        let action = cap_str(&caps, 1);
        let name = cap_opt(&caps, 4);
        let kind = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::ShrutiTrack { action, name, kind });
        }
    }

    if let Some(caps) = interp.try_captures("shruti_mixer", input_lower) {
        let track = cap_str(&caps, 1);
        let gain = caps
            .get(3)
            .and_then(|m| m.as_str().trim().parse::<f64>().ok());
        let mute = if caps.get(4).is_some() {
            Some(true)
        } else {
            None
        };
        let solo = if caps.get(5).is_some() {
            Some(true)
        } else {
            None
        };
        if !track.is_empty() {
            return Some(Intent::ShrutiMixer {
                track,
                gain,
                mute,
                solo,
            });
        }
    }

    if let Some(caps) = interp.try_captures("shruti_transport", input_lower) {
        let action = cap_str(&caps, 1);
        let value = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::ShrutiTransport { action, value });
        }
    }

    if let Some(caps) = interp.try_captures("shruti_export", input_lower) {
        let path = cap_opt(&caps, 1);
        let format = cap_opt(&caps, 3);
        return Some(Intent::ShrutiExport { path, format });
    }

    if let Some(caps) = interp.try_captures("shruti_plugins", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::ShrutiPlugins { action, name });
        }
    }
    if let Some(caps) = interp.try_captures("shruti_ai", input_lower) {
        let action = caps
            .get(1)
            .map_or("", |m| m.as_str())
            .trim()
            .replace(' ', "_")
            .to_string();
        let track = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::ShrutiAi { action, track });
        }
    }

    // --- Tazama video editor intents ---
    if let Some(caps) = interp.try_captures("tazama_project", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::TazamaProject { action, name });
        }
    }

    if let Some(caps) = interp.try_captures("tazama_timeline", input_lower) {
        let action = cap_str(&caps, 1);
        let clip_id = cap_opt(&caps, 4);
        let position = caps
            .get(6)
            .and_then(|m| m.as_str().trim().parse::<f64>().ok());
        if !action.is_empty() {
            return Some(Intent::TazamaTimeline {
                action,
                clip_id,
                position,
            });
        }
    }

    if let Some(caps) = interp.try_captures("tazama_effects", input_lower) {
        let action = cap_str(&caps, 1);
        let effect_type = cap_opt(&caps, 4);
        let clip_id = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::TazamaEffects {
                action,
                effect_type,
                clip_id,
            });
        }
    }

    if let Some(caps) = interp.try_captures("tazama_ai", input_lower) {
        let action = caps
            .get(1)
            .map_or("", |m| m.as_str())
            .trim()
            .replace(' ', "_")
            .to_string();
        let options = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::TazamaAi { action, options });
        }
    }

    if let Some(caps) = interp.try_captures("tazama_export", input_lower) {
        let path = cap_opt(&caps, 1);
        let format = cap_opt(&caps, 3);
        return Some(Intent::TazamaExport { path, format });
    }

    if let Some(caps) = interp.try_captures("tazama_media", input_lower) {
        let action = cap_str(&caps, 2);
        let path = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::TazamaMedia { action, path });
        }
    }
    if let Some(caps) = interp.try_captures("tazama_subtitles", input_lower) {
        let action = cap_str(&caps, 2);
        let language = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::TazamaSubtitles { action, language });
        }
    }

    // --- Rasa image editor intents ---
    if let Some(caps) = interp.try_captures("rasa_canvas", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::RasaCanvas { action, name });
        }
    }

    if let Some(caps) = interp.try_captures("rasa_layers", input_lower) {
        let action = cap_str(&caps, 1);
        let name = cap_opt(&caps, 4);
        let kind = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::RasaLayers { action, name, kind });
        }
    }

    if let Some(caps) = interp.try_captures("rasa_tools", input_lower) {
        let action = cap_str(&caps, 1);
        let params = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::RasaTools { action, params });
        }
    }

    if let Some(caps) = interp.try_captures("rasa_ai", input_lower) {
        let action = caps
            .get(1)
            .map_or("", |m| m.as_str())
            .trim()
            .replace(' ', "_")
            .to_string();
        let prompt = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::RasaAi { action, prompt });
        }
    }

    if let Some(caps) = interp.try_captures("rasa_export", input_lower) {
        let path = cap_opt(&caps, 1);
        let format = cap_opt(&caps, 3);
        return Some(Intent::RasaExport { path, format });
    }

    if let Some(caps) = interp.try_captures("rasa_batch", input_lower) {
        let action = cap_str(&caps, 1);
        let path = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::RasaBatch { action, path });
        }
    }
    if let Some(caps) = interp.try_captures("rasa_templates", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::RasaTemplates { action, name });
        }
    }
    if let Some(caps) = interp.try_captures("rasa_adjustments", input_lower) {
        let action = cap_str(&caps, 2);
        let adjustment_type = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::RasaAdjustments {
                action,
                adjustment_type,
            });
        }
    }

    // --- Mneme knowledge base intents ---
    if let Some(caps) = interp.try_captures("mneme_notebook", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::MnemeNotebook { action, name });
        }
    }

    if let Some(caps) = interp.try_captures("mneme_notes", input_lower) {
        let action = cap_str(&caps, 1);
        let title = cap_opt(&caps, 4);
        let notebook_id = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::MnemeNotes {
                action,
                title,
                notebook_id,
            });
        }
    }

    if let Some(caps) = interp.try_captures("mneme_search", input_lower) {
        let query = cap_str(&caps, 1);
        let mode = cap_opt(&caps, 3);
        if !query.is_empty() {
            return Some(Intent::MnemeSearch { query, mode });
        }
    }

    if let Some(caps) = interp.try_captures("mneme_ai", input_lower) {
        let action = caps
            .get(1)
            .map_or("", |m| m.as_str())
            .trim()
            .replace(' ', "_")
            .to_string();
        let note_id = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::MnemeAi { action, note_id });
        }
    }

    if let Some(caps) = interp.try_captures("mneme_graph", input_lower) {
        let action = caps
            .get(1)
            .map_or("", |m| m.as_str())
            .trim()
            .replace(' ', "_")
            .to_string();
        let node_id = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::MnemeGraph { action, node_id });
        }
    }

    if let Some(caps) = interp.try_captures("mneme_import", input_lower) {
        let action = cap_str(&caps, 1);
        let path = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::MnemeImport { action, path });
        }
    }
    if let Some(caps) = interp.try_captures("mneme_tags", input_lower) {
        let action = cap_str(&caps, 2);
        let tag = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::MnemeTags { action, tag });
        }
    }

    // --- Tarang media framework ---
    if let Some(caps) = interp.try_captures("tarang_probe", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::TarangProbe { path });
        }
    }
    if let Some(caps) = interp.try_captures("tarang_analyze", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::TarangAnalyze { path });
        }
    }
    if interp.try_captures("tarang_codecs", input_lower).is_some() {
        return Some(Intent::TarangCodecs);
    }
    if let Some(caps) = interp.try_captures("tarang_transcribe", input_lower) {
        let path = cap_str(&caps, 1);
        let language = cap_opt(&caps, 2);
        if !path.is_empty() {
            return Some(Intent::TarangTranscribe { path, language });
        }
    }
    if let Some(caps) = interp.try_captures("tarang_formats", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::TarangFormats { path });
        }
    }
    if let Some(caps) = interp.try_captures("tarang_fingerprint_index", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::TarangFingerprintIndex { path });
        }
    }
    if let Some(caps) = interp.try_captures("tarang_search_similar", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::TarangSearchSimilar { path, top_k: None });
        }
    }
    if let Some(caps) = interp.try_captures("tarang_describe", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::TarangDescribe { path });
        }
    }

    // --- Jalwa media player ---
    if let Some(caps) = interp.try_captures("jalwa_play", input_lower) {
        let path = cap_str(&caps, 1);
        if !path.is_empty() {
            return Some(Intent::JalwaPlay { path });
        }
    }
    if interp.try_captures("jalwa_pause", input_lower).is_some() {
        return Some(Intent::JalwaPause);
    }
    if interp.try_captures("jalwa_status", input_lower).is_some() {
        return Some(Intent::JalwaStatus);
    }
    if let Some(caps) = interp.try_captures("jalwa_search", input_lower) {
        let query = cap_str(&caps, 1);
        if !query.is_empty() {
            return Some(Intent::JalwaSearch { query });
        }
    }
    if let Some(caps) = interp.try_captures("jalwa_recommend", input_lower) {
        let item_id = cap_str(&caps, 1);
        let max = caps
            .get(2)
            .and_then(|m| m.as_str().trim().parse::<u32>().ok());
        if !item_id.is_empty() {
            return Some(Intent::JalwaRecommend { item_id, max });
        }
    }
    if let Some(caps) = interp.try_captures("jalwa_queue", input_lower) {
        let action = cap_str(&caps, 1);
        let item_id = cap_opt(&caps, 2);
        if !action.is_empty() {
            return Some(Intent::JalwaQueue { action, item_id });
        }
    }
    if let Some(caps) = interp.try_captures("jalwa_library", input_lower) {
        let action = cap_str(&caps, 1);
        let path = cap_opt(&caps, 2);
        if !action.is_empty() {
            return Some(Intent::JalwaLibrary { action, path });
        }
    }
    if let Some(caps) = interp.try_captures("jalwa_playlist", input_lower) {
        let action = cap_str(&caps, 1);
        let name = cap_opt(&caps, 2);
        let item_id = cap_opt(&caps, 3);
        if !action.is_empty() {
            return Some(Intent::JalwaPlaylist {
                action,
                name,
                item_id,
            });
        }
    }

    // --- Synapse LLM management intents ---
    if let Some(caps) = interp.try_captures("synapse_models", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        let source = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::SynapseModels {
                action,
                name,
                source,
            });
        }
    }

    if let Some(caps) = interp.try_captures("synapse_serve", input_lower) {
        let action = cap_str(&caps, 2);
        let model = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::SynapseServe { action, model });
        }
    }

    if let Some(caps) = interp.try_captures("synapse_finetune", input_lower) {
        let action = cap_str(&caps, 2);
        let model = cap_opt(&caps, 4);
        let method = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::SynapseFinetune {
                action,
                model,
                method,
            });
        }
    }

    if let Some(caps) = interp.try_captures("synapse_chat", input_lower) {
        let model = cap_str(&caps, 2);
        let prompt = cap_opt(&caps, 4);
        if !model.is_empty() {
            return Some(Intent::SynapseChat { model, prompt });
        }
    }

    if interp.try_captures("synapse_status", input_lower).is_some() {
        return Some(Intent::SynapseStatus);
    }

    if let Some(caps) = interp.try_captures("synapse_benchmark", input_lower) {
        let action = cap_str(&caps, 2);
        let models = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::SynapseBenchmark { action, models });
        }
    }
    if let Some(caps) = interp.try_captures("synapse_quantize", input_lower) {
        let action = cap_str(&caps, 2);
        let model = cap_opt(&caps, 4);
        let format = cap_opt(&caps, 6);
        if !action.is_empty() {
            return Some(Intent::SynapseQuantize {
                action,
                model,
                format,
            });
        }
    }

    None
}
