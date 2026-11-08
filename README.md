# Minecrafto-Rust (Java Edition -> Rust + WebAssembly + WebGL2)

Reimplementação *clean-room* da arquitetura e mecânicas do **Minecraft Java Edition** inteiramente em **Rust**, compilada diretamente para **WebAssembly (`wasm32-unknown-unknown`)** e renderizada no navegador com **WebGL2**.

---

## Mapeamento de Engenharia Reversa (Minecraft Java -> Rust)

| Subsistema | Classe Original (Minecraft Java Edition) | Módulo em Rust | Descrição Técnica |
| :--- | :--- | :--- | :--- |
| **Registro de Blocos** | `net.minecraft.world.level.tile.Tile` | [`src/blocks.rs`](src/blocks.rs) | 25 blocos + itens/picaretas, dureza (`block_hardness`), opacidade, emissão de luz e mapeamento de texturas por face (`FACE_TOP`..`FACE_WEST`). |
| **Ruído Perlin & Biomas** | `net.minecraft.world.level.levelgen.synth.ImprovedNoise` / `BiomeSource` | [`src/noise.rs`](src/noise.rs) | Curva quíntica $6t^5 - 15t^4 + 10t^3$, fBm 2D/3D multi-oitavas, geração de 4 biomas (*Plains*, *Forest*, *Desert*, *Snowy Mountains*) e cavernas 3D. |
| **Mundo, Chunks & Iluminação** | `Level`, `LevelChunk`, `ChunkRenderer` | [`src/world.rs`](src/world.rs) | Grade de $8 \times 8$ chunks ($128 \times 64 \times 128 = 1.048.576$ voxels) na memória linear do WASM, iluminação *Sky Light* + *Block Light*, *Smooth Ambient Occlusion (AO)* por vértice e árvores/minérios/TNT. |
| **Física & Colisão AABB** | `net.minecraft.world.phys.AABB` & `Entity.move` | [`src/physics.rs`](src/physics.rs) | Hitbox do jogador (`0.6 x 1.8`, olho em `1.62`), resolução de colisão por eixo (`Y -> X -> Z`), gravidade de `28 m/s²`, salto de `1.25` blocos, natação em água e modo voo. |
| **Raycasting Voxel 3D** | `Level.clip` (`HitResult`) | [`src/raycast.rs`](src/raycast.rs) | Algoritmo 3D DDA de Amanatides & Woo em Rust WASM com alcance de `5.5` blocos, retornando bloco atingido e normal da face para posicionamento. |
| **Sistema de Crafting** | `net.minecraft.item.crafting.CraftingManager` | [`src/crafting.rs`](src/crafting.rs) | Avaliador de receitas 2x2 e 3x3 em Rust WASM (Tábuas, Gravetos, Crafting Table, Fornalha, Picaretas de Madeira/Pedra/Ferro/Diamante, Vidro, Tijolos, Estante e TNT). |
| **Runtime WASM Zero-Dep** | `java.lang.*` / `core` | [`src/minicore.rs`](src/minicore.rs) | Runtime `#![no_core]` enxuto permitindo compilar o crate Rust diretamente para `wasm32-unknown-unknown` sem dependências externas. |
| **Exportações FFI WASM** | Cliente Principal | [`src/lib.rs`](src/lib.rs) | Funções `#[no_mangle] pub extern "C"` invocadas a 60 FPS pelo WebGL2 + suíte de testes unitários (`cargo test`). |

---

## Como Compilar e Rodar

### No Windows (PowerShell)

#### Opção A — Rodar Imediatamente (Sem precisar instalar o Rust)
O repositório já inclui o motor WebAssembly pré-compilado (`web/minecraft_rust.wasm`) embutido em `web/index.html` e `index.html`. No **Windows PowerShell**, basta clonar e iniciar o servidor na porta `8080`:

```powershell
git clone https://github.com/marlonferreira1800-hue/minecrafto-rust.git
cd minecrafto-rust
powershell -ExecutionPolicy Bypass -File .\start_windows.ps1
```

#### Opção B — Instalar o Rust no Windows, Recompilar o WASM e Rodar o Servidor Nativo
Se quiser compilar o código-fonte Rust (`src/*.rs`) e rodar o servidor HTTP nativo (`cargo run --release`):

```powershell
# 1. Instalar o Rust + Cargo no Windows via winget:
winget install Rustlang.Rustup

# 2. Recarregar o PATH na sessão atual do PowerShell e adicionar o target WebAssembly:
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
rustup target add wasm32-unknown-unknown

# 3. Entrar na pasta do projeto, compilar o WASM + rodar os testes e subir o servidor na porta 8080:
cd $HOME\minecrafto-rust
powershell -ExecutionPolicy Bypass -File .\build_wasm.ps1
cargo run --release
```

---

### No Linux / macOS / Git Bash

```bash
# 1. Compilar o motor Rust para WebAssembly (web/minecraft_rust.wasm), empacotar web/index.html e rodar os testes:
./build_wasm.sh

# 2. Iniciar o servidor HTTP nativo em Rust na porta 8080:
cargo run --release
```
