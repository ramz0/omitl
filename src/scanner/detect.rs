use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Framework {
    // Go
    Fiber,
    Gin,
    Echo,
    // Node
    Express,
    Fastify,
    // Python
    FastAPI,
    Flask,

    Unknown,
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Framework::Fiber   => write!(f, "Fiber (Go)"),
            Framework::Gin     => write!(f, "Gin (Go)"),
            Framework::Echo    => write!(f, "Echo (Go)"),
            Framework::Express => write!(f, "Express (Node.js)"),
            Framework::Fastify => write!(f, "Fastify (Node.js)"),
            Framework::FastAPI => write!(f, "FastAPI (Python)"),
            Framework::Flask   => write!(f, "Flask (Python)"),
            Framework::Unknown => write!(f, "Unknown"),
        }
    }
}

pub fn detect(path: &Path) -> Framework {
    // Go — check go.mod
    let go_mod = path.join("go.mod");
    if go_mod.exists() {
        let content = std::fs::read_to_string(&go_mod).unwrap_or_default();
        if content.contains("gofiber/fiber")  { return Framework::Fiber; }
        if content.contains("gin-gonic/gin")  { return Framework::Gin;   }
        if content.contains("labstack/echo")  { return Framework::Echo;  }
    }

    // Node — check package.json
    let pkg = path.join("package.json");
    if pkg.exists() {
        let content = std::fs::read_to_string(&pkg).unwrap_or_default();
        if content.contains("\"fastify\"") { return Framework::Fastify; }
        if content.contains("\"express\"") { return Framework::Express; }
    }

    // Python — check requirements.txt or pyproject.toml
    for file in &["requirements.txt", "pyproject.toml", "setup.py"] {
        let p = path.join(file);
        if p.exists() {
            let content = std::fs::read_to_string(&p).unwrap_or_default();
            if content.to_lowercase().contains("fastapi") { return Framework::FastAPI; }
            if content.to_lowercase().contains("flask")   { return Framework::Flask;   }
        }
    }

    Framework::Unknown
}
