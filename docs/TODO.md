# TODO — état de la session

Der dernier état du projet, pour reprendre dans un autre outil (cf.
DECISIONS.md d'agents-shared : la session ne voyage pas, les fichiers oui).

## État (2026-11-09)

- Squelette initial commité (`2e59765`, commit racine) : structure TEA
  (`model/message/update/view` + harnais), squelette hello world vérifié
  (build, run, test, clippy sans warning). Pas encore de dépendance
  externe — eframe, egui_kittest et ksni arrivent avec le premier UC
  (dernières versions stables, préférence durable).
- agents-shared poussé : profils `rust-t`, `rust-gui`, `ux` disponibles ;
  `preferences-dev.md` à lire en début de session.
- Templates UC/SR repris de rawtandem dans `docs/usecases/` — préfixe
  **SR** (*shared rule*) remplaçant « rc — règle commune ».

## Prochaine étape (la plume est à l'utilisateur)

1. Écrire le premier UC : `docs/usecases/uc-01-….md` (candidat naturel :
   « Mettre en hibernation au franchissement du seuil ») ;
2. arbitrer le terme candidat du glossaire : hibernation / suspension
   (faux ami, `docs/GLOSSAIRE.md`) ;
3. puis la boucle SPEC-UC (`docs/SPEC-UC.md`) : questionnement → tests
   rouges → relecture → go → TDD.

## UC à écrire (candidats)

- « Arrêter la surveillance » : suspendre la surveillance sans quitter
  (batterie en remplacement, machine branchée en permanence…) — touche la
  forme du modèle (état actif/inactif persisté ?) et l'UI du tray ;
  distinct de « Quitter l'application » (UC-05).

## Bugs de recette UC-05 (vu en recette visuelle, à corriger)

- Démarrage : une fenêtre apparaît brièvement puis disparaît (flash du
  premier rendu — eframe force l'affichage, le harnais recache au tick
  ≥ 2, cf. `hide_root_forever` dans `main.rs`).
- Organisation de la fenêtre de confirmation à challenger : appliquer
  réellement les préceptes UX/UI (`docs/UX-GENERIC.md`) — centrage,
  marges, hiérarchie visuelle, look de dialogue natif.
- La fenêtre de confirmation apparaît en haut à gauche de l'écran,
  pas au centre ; non conforme aux conventions de positionnement
  des dialogues — à corriger avec l'entrée précédente (placement
  `ViewportCommand`, cf. `egui::ViewportCommand::center_on_screen`).
- « Oui » ne quitte pas l'application. **Résolu** : drapeau « fermeture
  demandée par nous » dans le harnais (`self_close` dans `main.rs`) —
  winit renvoie notre propre `ViewportCommand::Close` sous forme
  d'évènement `Close` en entrée, indistinguable du ✕ de l'utilisateur ;
  le garde-fou « ✕ ≡ Non » ne l'annule plus lorsqu'il est marqué
  nôtre. Reproduit puis verrouillé dans la spec test
  (`tests/uc05_quitter_application.rs`, phase 6-bug), verte.
  Recette manuelle à refaire pour confirmation.

## Remarques en attente (issues de revue UC, non arbitrées)

À trancher avant ou pendant la rédaction des scénarios :

- UC-02 : échec de **lecture** de la batterie (trait `Batterie` en
  erreur : `/sys` absent, batterie débranchée) — non couvert par la
  minimal guarantee actuelle (seule l'hibernation impossible l'est).
- UC-03 : la success guarantee (« l'appli se lance au login ») n'est pas
  observable sans déconnexion/reconnexion — garantir plutôt l'artefact :
  fichier XDG autostart créé quand coché, supprimé quand décoché.
- UC-03 : idempotence — case déjà cochée / fichier déjà présent → ne
  rien réécrire.
- UC-05 : localisation de l'action « Quitter » — ✕ de la fenêtre vs
  entrée « Quitter » du menu tray (règle fermer-fenêtre ≠ quitter
  déjà actée dans UC-04 success guarantee).
- UC-05 : quitter = perte de la protection batterie — ARBITRÉ :
  confirmation demandée (étape 4 du scénario, extension 5a).
- UC-05 : libellés — le menu dit « Fermer Soporifique » alors que le
  titre de l'UC est « Quitter l'application » et que « fermer » désigne
  déjà le geste fenêtre ≠ quitter (UC-04) ; harmoniser + candidat
  glossaire « quitter ↔ quit ». À arbitrer.
- UC-05 : icône tray résiduelle après l'arrêt — observable testable
  pour la guarantee.
- Glossaire : candidat « journal » ↔ `log` — utilisé dans les guarantees
  de UC-02/03/04 sans emplacement/format définis.
- SR à écrire : emplacement/format du journal (`journal lisible par
  l'utilisateur` = où, quoi, quand) ; mécanisme de persistence commun
  UC-01/UC-03 (même fichier de config ?) ; intervalle de polling
  (déjà en question ouverte UC-02).
- UC-05 : clic **gauche** sur l'icône tray — comportement non spécifié
  (probablement : ouvrir la configuration) ; sera tranché avec
  l'implémentation de l'UC-01. Le menu tray sera partagé par Quitter
  (UC-05), Configuration (UC-04) et « Arrêter la surveillance » plus
  tard — candidate SR « structure du menu tray ».
- UC-05 : extension « échec de l'arrêt » (jouer la minimal guarantee)
  volontairement omise pour rester simple — première implémentation ;
  à ajouter si besoin en passe ultérieure.
- UC-01 : coquille « pré-éxistante » → « préexistante ».
- UC-04 : coquille « si il y a » → « s'il y a ».

## Décisions actées à conserver

- egui + egui_kittest, TEA imposée par la structure des fichiers ;
- tray via `ksni` (StatusNotifierItem/DBus) ;
- core pur : effets par traits injectés (`Batterie`, `Horloge`,
  `Hibernateur`) — la vraie hibernation est non couvrable en test, un mode
  essai (dry run) couvrira la même chaîne ; déclarés dans
  `docs/COVERAGE.md`.
- Pas de DDD complet : ubiquitous language uniquement (glossaire +
  libellés UI = termes du glossaire).
