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
- UC-05 : quitter = perte de la protection batterie — l'utilisateur
  doit-il être averti/confirmé ? À trancher dans le scénario.
- UC-05 : icône tray résiduelle après l'arrêt — observable testable
  pour la guarantee.
- Glossaire : candidat « journal » ↔ `log` — utilisé dans les guarantees
  de UC-02/03/04 sans emplacement/format définis.
- SR à écrire : emplacement/format du journal (`journal lisible par
  l'utilisateur` = où, quoi, quand) ; mécanisme de persistence commun
  UC-01/UC-03 (même fichier de config ?) ; intervalle de polling
  (déjà en question ouverte UC-02).
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
