//! Content type definitions
//! 
//! Supports multimodal content (text, images, audio, video, etc.)

use serde::{Deserialize, Serialize};

/// 多模态内容类型（支持文本、图像、音频、视频等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    /// 文本内容
    Text(String),
    /// 图像内容（URL + 可选描述）
    Image {
        url: String,
        caption: Option<String>,
    },
    /// 音频内容（URL + 可选转录文本）
    Audio {
        url: String,
        transcript: Option<String>,
    },
    /// 视频内容（URL + 可选摘要）
    Video {
        url: String,
        summary: Option<String>,
    },
    /// 结构化数据（JSON）
    Structured(serde_json::Value),
    /// 混合内容（多种类型组合）
    Mixed(Vec<Content>),
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_text())
    }
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Content::Text(a), Content::Text(b)) => a == b,
            (
                Content::Image {
                    url: u1,
                    caption: c1,
                },
                Content::Image {
                    url: u2,
                    caption: c2,
                },
            ) => u1 == u2 && c1 == c2,
            (
                Content::Audio {
                    url: u1,
                    transcript: t1,
                },
                Content::Audio {
                    url: u2,
                    transcript: t2,
                },
            ) => u1 == u2 && t1 == t2,
            (
                Content::Video {
                    url: u1,
                    summary: s1,
                },
                Content::Video {
                    url: u2,
                    summary: s2,
                },
            ) => u1 == u2 && s1 == s2,
            (Content::Structured(v1), Content::Structured(v2)) => v1 == v2,
            (Content::Mixed(m1), Content::Mixed(m2)) => m1 == m2,
            _ => false,
        }
    }
}

impl Content {
    /// 获取文本表示（用于向后兼容）
    pub fn as_text(&self) -> String {
        match self {
            Content::Text(s) => s.clone(),
            Content::Image { url, caption } => {
                format!(
                    "[Image: {}{}]",
                    url,
                    caption.as_ref().map(|c| format!(" - {c}")).unwrap_or_default()
                )
            }
            Content::Audio { url, transcript } => {
                format!(
                    "[Audio: {}{}]",
                    url,
                    transcript.as_ref().map(|t| format!(" - {t}")).unwrap_or_default()
                )
            }
            Content::Video { url, summary } => {
                format!(
                    "[Video: {}{}]",
                    url,
                    summary.as_ref().map(|s| format!(" - {s}")).unwrap_or_default()
                )
            }
            Content::Structured(v) => {
                serde_json::to_string(v).unwrap_or_else(|_| "[Structured Data]".to_string())
            }
            Content::Mixed(contents) => {
                contents.iter().map(|c| c.as_text()).collect::<Vec<_>>().join("\n")
            }
        }
    }

    /// 检查是否为文本内容
    pub fn is_text(&self) -> bool {
        matches!(self, Content::Text(_))
    }

    /// 获取纯文本（仅对 Text 类型有效）
    pub fn as_plain_text(&self) -> Option<&str> {
        match self {
            Content::Text(s) => Some(s),
            _ => None,
        }
    }

    /// 创建文本内容
    pub fn text<S: Into<String>>(s: S) -> Self {
        Content::Text(s.into())
    }
}
