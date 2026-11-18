// Point d'entree du frontend. L'etape 1 pose la fenetre principale avec le
// titre et un pied de page portant la version du binaire. Les grilles
// d'apprentissage et d'association arrivent a l'etape 2.

import { invoke } from "@tauri-apps/api/core";

async function monter() {
  const racine = document.getElementById("app");
  if (!racine) return;

  // La version est lue dans le binaire Rust (derivee par build.rs depuis le
  // tag git), pas dans le package.json ni dans le manifeste Cargo. Un seul
  // point de verite empeche le decalage entre ce qui tourne et ce qui
  // s'affiche.
  const version = await invoke<string>("version").catch(() => "inconnue");

  racine.innerHTML = `
    <header>
      <h1>magic-osc</h1>
      <p>Apprenez une surface MIDI à une console en OSC.</p>
    </header>
    <section id="apprentissage">
      <p>Interface d'apprentissage : à venir.</p>
    </section>
    <footer>
      <small>magic-osc ${version} — BREIZHZION</small>
    </footer>
  `;
}

monter();
