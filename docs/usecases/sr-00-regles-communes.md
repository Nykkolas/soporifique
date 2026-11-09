# Shared rules (règles communes)

Index des règles transversales, partagées par plusieurs UC — préfixe
**SR** (*shared rule*). Elles ne se répètent pas dans les scénarios : un
UC qui s'en réclame les référence dans son contexte
(`règles communes: SR-01, SR-02`) ou inline à une ou des étape(s)
particulière(s).

Leur couverture vit dans la suite de scénarios du projet (à créer au
premier UC, cf. `docs/SPEC-UC.md`) — un fichier par règle, cartouche de
chaque fichier : couverture et non couvert :

- ***coverage: générique*** : la règle a son propre test E2E dédié, valable
  pour toutes les instances qui s'en réclament. Il est écrit à la
  validation de la règle (rouge pour la bonne raison s'il y a du nouveau),
  relu par l'utilisateur avant l'implémentation.
- ***coverage: instancié*** : la règle est couverte par les tests de
  scénario des UC qui la référencent ; pas de test dédié. À la validation
  de la règle, vérifier que ces tests l'exercent ; sinon l'UC gagne
  l'étape/extension qui l'exerce, ou la règle passe en `générique`.
- Si rien n'est écrit, par défaut c'est `instancié`.

## Les règles

| Règle | Fichier |
|---|---|
