# Historique de couverture

Une ligne par mesure (voir docs/COVERAGE-GENERIC.md pour la procédure).
La dernière ligne est la référence pour le calcul des écarts.

Non couvert structurel (déclaré une bonne fois) : la vraie hibernation
n'est pas rejouable en test — recette manuelle seulement ; le mode essai
(dry run) couvrira la même chaîne côté tests ; le harnais (`main.rs` et
`tray.rs` : câblage eframe/ksni lancé par l'utilisateur) n'est pas
couvert en test.

| Date | Changement | Plateforme | Couverture globale | Par module (delta) | Notes |
|---|---|---|---|---|---|
| 2026-11-09 | Implémentation UC-05 (scénario primaire : confirmation d'arrêt) — première mesure | Linux, `cargo-llvm-cov` 0.9.1 | 95,8 % (lignes) | update.rs 100 % ; view.rs 95,7 % ; main.rs 0 % | Première mesure, pas d'écart. view.rs : 1 ligne manquante = bouton « Non » (extension 5a, test à venir). main.rs : harnais, plafond structurel déclaré ci-dessus. |
| 2026-11-09 | Harnais de recette UC-05 : fenêtre eframe + icône tray ksni | Linux, `cargo-llvm-cov` 0.9.1 | 35,4 % (lignes) | cœur TEA inchangé : update.rs 100 %, view.rs 95,7 % ; main.rs 0 %, tray.rs 0 % | Écart −60 pts : +123 lignes de harnais (main.rs + tray.rs), plafond structurel déclaré ci-dessus (câblage UI/DBus non rejouable en test). Recette visuelle à faire par l'utilisateur. |
| 2026-11-09 | Bug recette UC-05 « Oui ne quitte pas » : drapeau self-close ; vue = dialogue (fenêtre unique à la demande) | Linux, `cargo-llvm-cov` 0.9.1 | 37,5 % (lignes) | cœur TEA : update.rs 100 %, view.rs 95,7 % ; main.rs 0 % (+23 lignes), tray.rs 0 % | Écart +2 pts. Nouvelles lignes en harnais (plafond structurel) et spec test (non comptée) ; la ligne manquante de view.rs reste le bouton « Non » (extension 5a). Reproduction du bug verrouillée dans la spec test, puis verte. |
| 2026-11-09 | Refactoring shell : runtime pur extrait dans la lib (décisions → effets), main.rs réduit à la colle ; spec test sur le vrai code | Linux, `cargo-llvm-cov` 0.9.1 | 61,8 % (lignes) | lib : update.rs 100 %, runtime.rs 97,5 %, view.rs 95,7 % ; binaire : main.rs 0 % (62 lignes de colle), tray.rs 0 % | Écart +24 pts : la logique du shell (visibilité de fenêtre, garde ✕ ≡ Non, drapeau self_close) est passée du binaire non couvrable à la lib testée. Cœur lib (update+runtime+view) : 97,9 %. main/tray = plafond structurel (vraie colle egui/DBus). Attention : `--lib` seul sous-estime le cœur (la spec E2E vit dans `tests/` — view 0 % sans elle) : mesurer avec `--all-targets` et lire les modules lib ligne à ligne. |
| 2026-11-09 | Extension 5a UC-05 sous test (bouton « Non ») | Linux, `cargo-llvm-cov` 0.9.1 | 62,2 % (lignes) | lib : update.rs 100 %, view.rs **100 %**, runtime.rs 97,5 % ; binaire : main.rs/tray.rs 0 % (plafond structurel) | Écart +0,4 pt : la dernière ligne manquante (bouton « Non ») est couverte — view.rs à 100 %. Test vérifié rouge pour les bonnes raisons par mutations (swap Oui/Non attrapé sur « jamais de fermeture ordonnée » ; message avalé attrapé sur « la demande disparaît »), puis revenu au vert sans toucher aux assertions. Restent 3 lignes de runtime.rs (branches de garde). |
