// legixy-core: 共通型・エラー階層・識別子（ADR-LGX-020）
// 本ファイルは TC[RED] フェーズの scaffold。共有型のシグネチャを凍結 DD に整合させ、
// 実体ロジックは SRC[GREEN] で実装する。

use serde::{Deserialize, Serialize};

/// 検証済み識別子（書式 `{type}-{area}-{seq}` または `{id}#{subnode_hash}`）。
/// DD-LGX-001 §2.1 / ADR-LGX-021 §2.1（グラフキー `NodeId = String` とは区別する書式検証用 newtype）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id(pub String);

impl Id {
    pub fn new(s: impl Into<String>) -> Self {
        Id(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// finding の重大度（DD-LGX-001 §2.2 / SPEC-LGX-004.REQ.03）。Ok は予約（finding 非発行）。
/// 宣言順 = 昇順（Ok < Info < Warning < Error）。REQ.06 の severity 降順ソートは Reverse で行う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Ok,
    Info,
    Warning,
    Error,
}

/// area 別 chain 定義（multi-area、`[[id.chains]]`。DevProc_V4.1 §12 / SPEC-LGX-008.REQ.03a）。
#[derive(Debug, Clone)]
pub struct ChainSpec {
    pub area: String,
    pub order: Vec<String>,
    pub independent: Vec<String>,
}

/// typecode → ディレクトリ / 拡張子（`[id.types]`。OrphanFile 走査の底本）。
#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub typecode: String,
    pub dir: String,
    pub ext: String,
}

/// 意味層（第 2 層）設定（`[semantic]`、LGX-COMPAT-001 §6）。閾値は config 由来。
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    pub enabled: bool,
    pub model: Option<String>,
    pub model_dir: Option<String>,
    pub similarity_threshold: f32,
    pub drift_threshold: f32,
    pub link_candidate_threshold: f32,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        SemanticConfig {
            enabled: false,
            model: None,
            model_dir: None,
            similarity_threshold: 0.4,
            // 既定 investigate 剪定値（旧 CLI の DEFAULT_DRIFT_THRESHOLD=0.25 連続性。config 指定で上書き）。
            drift_threshold: 0.25,
            link_candidate_threshold: 0.7,
        }
    }
}

/// 設定（`.legixy.toml` / `.trace-engine.toml` 由来。LGX-COMPAT-001 §6 / SPEC-LGX-008.REQ.13）。
#[derive(Debug, Clone)]
pub struct Config {
    /// chain typecode の順序（単一 area。`[id.chain] order`。ChainIntegrity 検査の底本）。
    pub chain_order: Vec<String>,
    /// chain 外 typecode（`[id.chain] independent`）。
    pub independent: Vec<String>,
    /// multi-area chain（`[[id.chains]]`）。空なら単一 area モード（chain_order を使う）。
    pub chains: Vec<ChainSpec>,
    /// typecode → dir/ext（`[id.types]`。OrphanFile 走査に使用。空なら走査スキップ）。
    pub types: Vec<TypeSpec>,
    /// IdRedefined / IdSemanticMismatch / IdSemanticDrift 等の opt-in 検査（既定 false、REQ.11/12/13）。
    pub id_redefined_enabled: bool,
    pub id_semantic_mismatch_enabled: bool,
    pub id_semantic_drift_enabled: bool,
    /// 意味層設定（閾値・モデル）。
    pub semantic: SemanticConfig,
    /// `[contextual_retrieval] enabled`（LGX-EXT-002）。
    pub contextual_retrieval_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // ICONIX 二段化 chain（CLAUDE.md / .trace-engine.toml 既定）。
            chain_order: ["UC", "RBA", "SEQA", "RBD", "SEQD", "DD", "TS", "TC", "SRC"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            independent: Vec::new(),
            chains: Vec::new(),
            types: Vec::new(),
            id_redefined_enabled: false,
            id_semantic_mismatch_enabled: false,
            id_semantic_drift_enabled: false,
            semantic: SemanticConfig::default(),
            contextual_retrieval_enabled: false,
        }
    }
}

/// `Config::load` の結果。`source=None` は設定ファイル不在（既定使用）。
#[derive(Debug, Clone)]
pub struct ConfigLoad {
    pub config: Config,
    /// 読み込んだ設定ファイルのパス（不在時 None）。
    pub source: Option<std::path::PathBuf>,
    /// 旧名 `.trace-engine.toml` を fallback として読んだ場合 true（移行 Info の根拠）。
    pub legacy_fallback: bool,
}

impl Config {
    /// 設定ファイルを探索・解析する（SPEC-LGX-008.REQ.13）。
    /// 探索順 `.legixy.toml` → `.trace-engine.toml`。両在は前者優先、旧名のみは fallback（移行 Info）。
    /// いずれも不在なら既定（`source=None`）。**不正 TOML は `Err`（実行時失敗 = exit 1、EXT-ERR-003）**。
    pub fn load(root: &std::path::Path) -> Result<ConfigLoad, ConfigError> {
        let canonical = root.join(".legixy.toml");
        let legacy = root.join(".trace-engine.toml");
        let (path, is_legacy) = if canonical.exists() {
            (canonical, false)
        } else if legacy.exists() {
            (legacy, true)
        } else {
            return Ok(ConfigLoad {
                config: Config::default(),
                source: None,
                legacy_fallback: false,
            });
        };
        let text = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::Load(format!("{}: {e}", path.display())))?;
        let value: toml::Value = toml::from_str(&text)
            .map_err(|e| ConfigError::Load(format!("{} の TOML 解析に失敗: {e}", path.display())))?;
        Ok(ConfigLoad {
            config: Config::from_toml_value(&value),
            source: Some(path),
            legacy_fallback: is_legacy,
        })
    }

    /// 解析済み TOML から既知キーを抽出して Config を構築（未知キーは無視。スキーマは両ファイル共通）。
    fn from_toml_value(v: &toml::Value) -> Config {
        let mut c = Config::default();
        let id = v.get("id");
        // [id.chain] order / independent（単一 area）
        if let Some(chain) = id.and_then(|i| i.get("chain")) {
            if let Some(order) = chain.get("order").and_then(|o| o.as_array()) {
                c.chain_order = string_array(order);
            }
            if let Some(indep) = chain.get("independent").and_then(|o| o.as_array()) {
                c.independent = string_array(indep);
            }
        }
        // [[id.chains]]（multi-area）
        if let Some(chains) = id.and_then(|i| i.get("chains")).and_then(|c| c.as_array()) {
            c.chains = chains
                .iter()
                .filter_map(|ch| {
                    let area = ch.get("area")?.as_str()?.to_string();
                    let order = string_array(ch.get("order")?.as_array()?);
                    let independent = ch
                        .get("independent")
                        .and_then(|o| o.as_array())
                        .map(|a| string_array(a))
                        .unwrap_or_default();
                    Some(ChainSpec {
                        area,
                        order,
                        independent,
                    })
                })
                .collect();
        }
        // [id.types]（typecode → dir/ext。テーブル形式・インラインテーブル形式の双方を受理）
        if let Some(types) = id.and_then(|i| i.get("types")).and_then(|t| t.as_table()) {
            for (typecode, spec) in types {
                if let Some(dir) = spec.get("dir").and_then(|x| x.as_str()) {
                    let ext = spec
                        .get("ext")
                        .and_then(|x| x.as_str())
                        .unwrap_or(".md")
                        .to_string();
                    c.types.push(TypeSpec {
                        typecode: typecode.clone(),
                        dir: dir.to_string(),
                        ext,
                    });
                }
            }
        }
        // [semantic]
        if let Some(s) = v.get("semantic") {
            if let Some(b) = s.get("enabled").and_then(|x| x.as_bool()) {
                c.semantic.enabled = b;
            }
            if let Some(m) = s.get("model").and_then(|x| x.as_str()) {
                c.semantic.model = Some(m.to_string());
            }
            if let Some(m) = s.get("model_dir").and_then(|x| x.as_str()) {
                c.semantic.model_dir = Some(m.to_string());
            }
            if let Some(t) = s.get("similarity_threshold").and_then(as_f32) {
                c.semantic.similarity_threshold = t;
            }
            if let Some(t) = s.get("drift_threshold").and_then(as_f32) {
                c.semantic.drift_threshold = t;
            }
            if let Some(t) = s.get("link_candidate_threshold").and_then(as_f32) {
                c.semantic.link_candidate_threshold = t;
            }
        }
        // [contextual_retrieval]
        if let Some(cr) = v.get("contextual_retrieval") {
            if let Some(b) = cr.get("enabled").and_then(|x| x.as_bool()) {
                c.contextual_retrieval_enabled = b;
            }
        }
        c
    }

    /// node id（`{type}-{area}-{seq}`）の area に対応する chain order を返す。
    /// multi-area（chains 非空）は area 解決、単一 area モードは chain_order を返す。
    /// 解決できない area（独立／未知）は None。
    pub fn chain_order_for(&self, node_id: &str) -> Option<&Vec<String>> {
        if self.chains.is_empty() {
            return Some(&self.chain_order);
        }
        let area = node_id.split('-').nth(1)?;
        self.chains
            .iter()
            .find(|ch| ch.area == area)
            .map(|ch| &ch.order)
    }
}

fn string_array(a: &[toml::Value]) -> Vec<String> {
    a.iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect()
}

fn as_f32(v: &toml::Value) -> Option<f32> {
    v.as_float()
        .map(|f| f as f32)
        .or_else(|| v.as_integer().map(|i| i as f32))
}

/// 設定ロード失敗（実行時失敗 = exit 1）。
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config load error: {0}")]
    Load(String),
}
