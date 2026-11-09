# Soporifique 🌙

Soporifique veille sur la batterie de votre portable : quand le niveau
franchit le seuil que vous avez choisi, la machine entre en hibernation
— endormez-vous, il veille.

> **Statut : développement précoce.** Seul « Quitter l'application »
> (UC-05) est implémenté — la surveillance de batterie et l'hibernation
> arrivent avec les UC-01/02. Le détail vit dans `docs/TODO.md`.

## Ce que l'application fait aujourd'hui

- une icône en croissant de lune dans la barre d'état système (zone de
  notification, Linux uniquement) ;
- clic droit → « Quitter Soporifique » → une fenêtre de confirmation
  (« Voulez-vous vraiment quitter Soporifique ? … ») ;
- « Oui » quitte, « Non » (ou fermer la fenêtre) ne quitte jamais.

## Ce qu'elle fera

Défini par ses use cases (`docs/usecases/`) : régler le seuil de
batterie minimum (UC-01), hiberner au franchissement du seuil (UC-02),
lancer Soporifique au login (UC-03), accéder à la configuration (UC-04),
et un jour suspendre la surveillance sans quitter.

## Prérequis et lancement

- Linux avec une session graphique et un gestionnaire de zone de
  notification (testé sur Linux Mint / Cinnamon) ;
- Rust (édition 2024).

```
cargo run
```

Les tests et la couverture :

```
cargo test          # tests unitaires + spec E2E headless
cargo clippy        # zéro warning exigé
cargo llvm-cov      # historique dans docs/COVERAGE.md
```

## Documentation

- `AGENTS.md` : règles du projet, mode de collaboration ;
- `docs/usecases/` : les use cases (la spec est le UC) ;
- `docs/GLOSSAIRE.md` : langage omniprésent (terme FR ↔ code) ;
- `docs/COVERAGE.md` : historique de couverture mesure par mesure ;
- `docs/TODO.md` : état de la session, bugs en attente, candidats.
