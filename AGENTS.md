# AGENTS.md - Modèle (agents-shared)

<!--
Modèle d'AGENTS.md pour un nouveau projet. Copié par init-preferences.sh,
il ne remplace jamais un AGENTS.md existant.
Adapter les sections marquées [À ADAPTER]. Ne pas supprimer les sections
« Mode de collaboration » et « Règles partagées ».
-->

# AGENTS.md - Règles et meilleures pratiques pour ce projet

## 🤝 Mode de collaboration

### Préférences durables de l'utilisateur

Lire `../agents-shared/preferences-dev.md` : c'est l'utilisateur qui décide
ce qui est développé ensuite ; boucle spec → tests rouges → relecture →
implémentation ; ubiquitous language ; GUI testable headless ; TEA ;
design with types ; couverture mesurée dès que tout est vert.

### Commits et push
- Ne jamais commiter ni pousser sans l'**autorisation explicite** de l'utilisateur : proposer, attendre le oui.

### "Propose-moi" = proposer, ne pas exécuter
- Quand l'utilisateur demande de **proposer** (améliorations, changements, refactor), NE PAS modifier les fichiers.
- Expliquer d'abord ce qui est prévu : fichiers concernés, commandes à exécuter, risques.
- Attendre l'**autorisation explicite** de l'utilisateur avant d'appliquer quoi que ce soit.
- Les demandes directes ("fais", "corrige", "ajoute") restent des autorisations à agir immédiatement.

### "Audit / review du code"
- Lire `docs/AUDIT.md` (checklist spécifique) et `docs/AUDIT-GENERIC.md` (procédure, thèmes, sévérités, format de rapport).
- L'audit ne modifie aucun fichier : recommandations uniquement, sauf demande explicite de corriger.

### "Crée une version x.y.z"
- Équivaut à : `git tag vx.y.z` + `git push origin vx.y.z` (autorisation explicite incluse).
- **Description avant le tag** : quand l'utilisateur demande une version,
  l'agent rédige et propose un **court texte de description** (langage utilisateur : ce que
  la version apporte, pas un changelog) **avant de créer le tag** ; sans validation explicite
  du texte par l'utilisateur, pas de tag. Le texte validé rejoint la Release (corps,
  notes générées en complément).
- **Publication toujours manuelle** : quelle que soit la formulation de la demande
  (« crée », « publie »…), l'agent s'arrête au brouillon de Release — il ne publie
  **jamais** ; c'est l'utilisateur qui publie après relecture.
[À ADAPTER] Workflow CI : pas de CI pour l'instant (dépôt local). À compléter si le projet est publié.

### Workflow « la spec est le UC »
- La boucle par défaut pour toute fonctionnalité ou correction vit dans `docs/SPEC-UC.md` :
  UC écrit par l'utilisateur → questionné par l'agent (sans la plume) → test E2E relu par
  l'utilisateur → « go » → implémentation TDD → vérification utilisateur → commit.
- Ne pas démarrer une fonctionnalité sans UC validé, ne pas implémenter sans le « go ».

## 🌍 Langue

- **Code en anglais** : identifiants, fonctions, modules, messages, noms de tests.
- **UI en français** : libellés, statuts, messages affichés à l'utilisateur.
- La documentation (AGENTS.md, AUDIT.md, TODO.md, README) est en français.

## 📚 Règles partagées (agents-shared, à lire selon le contexte)

Ces fichiers sont des symlinks vers `~/Source/agents-shared/` :

- **Rust / TEA** : lire `docs/RUST-TEA.md` avant d'écrire ou de modifier du Rust.
- **egui / tray** : lire `docs/RUST-EGUI.md` avant de toucher l'UI, les tests headless ou le tray.
- **UX/UI** : `docs/UX-GENERIC.md`, règle par défaut pour toute interface ; la prime sur une « bonne idée » locale est à la convention de plateforme.
- **Audit générique** : `docs/AUDIT-GENERIC.md`, complété par la checklist locale `docs/AUDIT.md`.
- **Erreurs transverses** : `docs/ERREURS-GENERIC.md`, leçons génériques à ne pas reproduire ; y promouvoir les leçons locales génériques (procédure dans DECISIONS.md d'agents-shared).
- **Spec = UC** : `docs/SPEC-UC.md`, workflow de spécification par use cases (la boucle complète y vit).
- **Couverture de code** : `docs/COVERAGE-GENERIC.md` (mesure systématique quand tout est vert, écart rapporté depuis la mesure précédente, historique dans `docs/COVERAGE.md`).

## 🏛️ Architecture

**The Elm Architecture (TEA)** imposée par la structure des fichiers (détails dans `docs/RUST-TEA.md` et `docs/RUST-EGUI.md`) :

- `src/model.rs` : l'état, immuable, mis à jour uniquement par `update`.
- `src/message.rs` : l'enum exhaustive des Messages (un par geste, tous producteurs confondus : UI egui, tray ksni, timer de polling).
- `src/update.rs` : fonction pure `update(Model, Message) -> (Model, Vec<Command>)` — la garde des actions vit ici, jamais dans les callbacks de vue.
- `src/view.rs` : rend le modèle, émet des Messages, ne fait rien d'autre.
- `src/main.rs` : le harnais (câblage egui/ksni, exécution des Command).

Frontières à respecter :

- Le **domaine** (niveau de batterie, seuil, décision de franchissement) ne connaît ni egui ni /sys ni systemctl : les effets extérieurs arrivent par traits injectés (`Batterie`, `Horloge`, `Hibernateur`) — le cœur reste pur et testable sans écran ni batterie réelle.
- **Ubiquitous language** : le vocabulaire du domaine vit dans `docs/GLOSSAIRE.md` (terme FR ↔ identifiant de code) ; les libellés UI français sont des termes du glossaire, regroupés dans le module `labels` ; l'agent relève les candidats, l'utilisateur arbitre et rédige.
- **`docs/usecases/`** : UC Cockburn léger (`uc-template.md`) et règles partagées **SR** — *shared rule*, équivalent anglais des « rc » de rawtandem (`sr-template.md`, index `sr-00-regles-communes.md`). La plume reste à l'utilisateur.
- **Non couvert permanent** (à rappeler dans chaque rapport de couverture) : la vraie hibernation n'est pas rejouable en test — vérifiée en recette manuelle seulement ; un mode essai (dry run) couvrira la même chaîne côté tests.
- **Logique testable vs colle non testable** (préférence durable n° 8, `../agents-shared/preferences-dev.md`) : séparer au maximum la logique métier/applicative testable des fonctions non testables (affichage, communications avec des tiers). Toute décision vit dans la lib testée (`update`, `runtime`) et retourne des effets/Command explicites ; le binaire (`main.rs`, `tray.rs`) n'est que de la colle les exécutant — aucune décision dedans. Une décision logée dans le harnais est invisible aux tests et n'apparaît qu'en recette manuelle.

## Bonnes pratiques générales

### TDD
- Approche par défaut pour les corrections et nouvelles fonctionnalités : écrire d'abord le test qui échoue, le voir échouer, puis corriger le code, puis revérifier (rouge/vert).
- Chaque bug corrigé doit avoir son test de non-régression.
- Quand toute la suite est verte en fin de changement : mesurer la couverture et rapporter l'écart depuis la mesure précédente (`docs/COVERAGE-GENERIC.md`, historique dans `docs/COVERAGE.md`).

### DRY
- Éviter la répétition de code. Extraire les conditions répétées en fonctions helpers.

### Strings magiques
- Éviter les strings littérales dans le code. Préférer des constantes ou des types dédiés.

### Nommage
- **Messages** : des noms explicites (`...Requested`, `...Finished`, `...Failed`).
- **State** : des noms clairs (`IsBusy`, `Status`).
- **Fonctions** : des verbes (`findPairs`, `copyText`).

## ❌ Erreurs à ne pas reproduire

Spécifiques à ce projet : aucune pour l'instant (vierge). Les leçons
génériques vivent dans `docs/ERREURS-GENERIC.md` ; les pièges spécifiques
Rust/egui dans `docs/RUST-TEA.md` et `docs/RUST-EGUI.md`. Toute leçon
apprise ici est notée (avec le contexte qui l'a révélée) et promue vers
agents-shared si elle est générique.
