# AUDIT.md - Grille d'audit du code

<!--
Modèle créé par init-preferences.sh. Il ne remplace jamais un AUDIT.md existant.
La procédure, les sévérités, les thèmes génériques et le format du rapport sont
dans docs/AUDIT-GENERIC.md (agents-shared). Ce fichier ne contient que la
checklist SPÉCIFIQUE au projet.
-->

Procédure à suivre quand l'utilisateur demande un audit / une review du code :
suivre `docs/AUDIT-GENERIC.md`, puis passer la checklist spécifique ci-dessous.
L'audit ne modifie **aucun fichier**.

## 🎯 Checklist spécifique à ce projet

[À ADAPTER] Exemples de formulation par domaine :

- [ ] Sécurité : [ex. aucune fuite de secret ; données sensibles jamais loggées]
- [ ] Opérations fichiers : [ex. suppression uniquement via la corbeille OS ; confirmation explicite avant toute opération destructive]
- [ ] Cas limites du domaine : [ex. entrées vides, orphelins, collisions de noms]

## 📄 Format du rapport

Voir `docs/AUDIT-GENERIC.md` : tableau unique trié par sévérité, conclusion
d'une à trois phrases.
