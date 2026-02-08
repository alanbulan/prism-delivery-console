// ============================================================================
// LLM 客户端服务：与 OpenAI 兼容 API 通信
// ✅ 只能做：HTTP 请求、JSON 解析
// ⛔ 禁止：依赖 tauri::*，直接操作数据库
// ============================================================================

use serde::{Deserialize, Serialize};

/// OpenAI /v1/models 响应结构
#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

/// 单个模型条目
#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

/// Chat Completion 请求体
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

/// Chat 消息
#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Chat Completion 响应体
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

/// Chat 选项
#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

/// Chat 响应消息
#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

/// 从 OpenAI 兼容 API 获取可用模型列表
///
/// # 参数
/// - `base_url`: API 基础地址（如 http://localhost:11434/v1）
/// - `api_key`: API Key（可为空字符串）
///
/// # 返回
/// - `Ok(Vec<String>)`: 模型 ID 列表
/// - `Err(String)`: 请求失败的错误描述
pub async fn fetch_models(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    // 拼接 /models 端点，兼容末尾有无斜杠
    let url = format!("{}/models", base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let mut req = client.get(&url);

    // 如果提供了 API Key，添加 Authorization 头
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求模型列表失败：{}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "获取模型列表失败：HTTP {}",
            resp.status()
        ));
    }

    let body = resp
        .json::<ModelsResponse>()
        .await
        .map_err(|e| format!("解析模型列表响应失败：{}", e))?;

    let model_ids: Vec<String> = body.data.into_iter().map(|m| m.id).collect();
    Ok(model_ids)
}

/// 调用 OpenAI 兼容 Chat Completion API 生成文件摘要
///
/// # 参数
/// - `base_url`: API 基础地址
/// - `api_key`: API Key（可为空）
/// - `model`: 模型名称
/// - `file_path`: 文件相对路径（用于 prompt 上下文）
/// - `file_content`: 文件内容
///
/// # 返回
/// - `Ok(String)`: LLM 生成的摘要文本
/// - `Err(String)`: 请求失败的错误描述
pub async fn generate_summary(
    base_url: &str,
    api_key: &str,
    model: &str,
    file_path: &str,
    file_content: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // 截断过长的文件内容，避免超出 token 限制
    let max_chars = 8000;
    let content = if file_content.len() > max_chars {
        &file_content[..max_chars]
    } else {
        file_content
    };

    let request_body = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "你是一个代码分析助手。请用简洁的中文对给定的源代码文件进行摘要，包括：1) 文件的主要职责 2) 关键的函数/类/接口 3) 依赖关系。摘要控制在 200 字以内。".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("请分析以下文件：\n\n文件路径：{}\n\n```\n{}\n```", file_path, content),
            },
        ],
        temperature: 0.3,
    };

    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&request_body);

    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("调用 LLM API 失败：{}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("LLM API 返回错误：HTTP {} - {}", status, body_text));
    }

    let chat_resp = resp
        .json::<ChatResponse>()
        .await
        .map_err(|e| format!("解析 LLM 响应失败：{}", e))?;

    chat_resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "LLM 返回了空的 choices".to_string())
}

// ============================================================================
// Embedding 生成
// ============================================================================

/// Embedding 请求体（OpenAI 兼容 /v1/embeddings）
#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

/// Embedding 响应体
#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

/// 单个 Embedding 数据
#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

/// 调用 OpenAI 兼容 Embedding API 生成文本向量
///
/// # 参数
/// - `base_url`: API 基础地址
/// - `api_key`: API Key（可为空）
/// - `model`: Embedding 模型名称（如 nomic-embed-text）
/// - `text`: 要生成向量的文本
///
/// # 返回
/// - `Ok(Vec<f32>)`: 向量数组
/// - `Err(String)`: 请求失败的错误描述
pub async fn generate_embedding(
    base_url: &str,
    api_key: &str,
    model: &str,
    text: &str,
) -> Result<Vec<f32>, String> {
    let url = format!("{}/embeddings", base_url.trim_end_matches('/'));

    let request_body = EmbeddingRequest {
        model: model.to_string(),
        input: text.to_string(),
    };

    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&request_body);

    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("调用 Embedding API 失败：{}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("Embedding API 返回错误：HTTP {} - {}", status, body_text));
    }

    let emb_resp = resp
        .json::<EmbeddingResponse>()
        .await
        .map_err(|e| format!("解析 Embedding 响应失败：{}", e))?;

    emb_resp
        .data
        .into_iter()
        .next()
        .map(|d| d.embedding)
        .ok_or_else(|| "Embedding API 返回了空的 data".to_string())
}

/// 调用 LLM 生成项目分析报告（通用 Chat Completion）
///
/// # 参数
/// - `base_url`: API 基础地址
/// - `api_key`: API Key
/// - `model`: 模型名称
/// - `system_prompt`: 系统提示词
/// - `user_prompt`: 用户提示词（包含项目数据）
///
/// # 返回
/// - `Ok(String)`: LLM 生成的 Markdown 报告
pub async fn generate_report(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let request_body = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.3,
    };

    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&request_body);

    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("调用 LLM API 失败：{}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("LLM API 返回错误：HTTP {} - {}", status, body_text));
    }

    let chat_resp = resp
        .json::<ChatResponse>()
        .await
        .map_err(|e| format!("解析 LLM 响应失败：{}", e))?;

    chat_resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "LLM 返回了空的 choices".to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_url_trailing_slash_handling() {
        // 验证 URL 拼接逻辑（不发起实际请求）
        let base = "http://localhost:11434/v1/";
        let url = format!("{}/models", base.trim_end_matches('/'));
        assert_eq!(url, "http://localhost:11434/v1/models");

        let base2 = "http://localhost:11434/v1";
        let url2 = format!("{}/models", base2.trim_end_matches('/'));
        assert_eq!(url2, "http://localhost:11434/v1/models");
    }
}
