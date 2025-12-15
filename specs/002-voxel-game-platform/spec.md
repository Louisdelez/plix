# Feature Specification: Voxel Game Platform - Visual Multiplayer

**Feature Branch**: `002-voxel-game-platform`
**Created**: 2025-12-14
**Status**: Draft
**Input**: Transformer le prototype en plateforme voxel jouable avec rendu d'arène et visualisation multijoueur

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visualisation de l'arène voxel (Priority: P1)

En tant que développeur, je veux voir l'arène voxel affichée dans le client pour valider que le format d'arène et la géométrie fonctionnent correctement.

**Why this priority**: Sans rendu de l'arène, aucune autre fonctionnalité visuelle ne peut être testée. C'est le fondement de toute la visualisation du jeu.

**Independent Test**: Peut être testé en lançant le client windowed seul et vérifiant que les blocs de l'arène sont visibles avec une géométrie cohérente (sols, murs, volumes).

**Acceptance Scenarios**:

1. **Given** un client démarré en mode windowed, **When** l'arène est chargée, **Then** les blocs de l'arène sont visibles à l'écran avec des sols et murs distincts.
2. **Given** un client affichant l'arène, **When** je déplace la caméra avec WASD et la souris, **Then** je peux explorer toute la géométrie de l'arène de manière fluide.
3. **Given** un client affichant l'arène, **When** je regarde les volumes (murs, sols), **Then** ils sont visuellement cohérents et distinguables les uns des autres.

---

### User Story 2 - Visualisation des joueurs multijoueur (Priority: P2)

En tant que joueur/testeur, je veux voir les autres joueurs bouger de façon fluide dans l'arène pour valider que l'interpolation réseau fonctionne.

**Why this priority**: Une fois l'arène visible, voir les autres joueurs est essentiel pour valider la boucle multijoueur. L'interpolation fluide démontre que le réseau fonctionne correctement.

**Independent Test**: Peut être testé en connectant 2 clients au même serveur et vérifiant que chaque client voit l'autre joueur se déplacer sans téléportation.

**Acceptance Scenarios**:

1. **Given** un serveur en cours d'exécution et un client connecté, **When** un second client se connecte, **Then** le premier client voit une représentation visuelle du second joueur apparaître.
2. **Given** deux clients connectés au même serveur, **When** un joueur se déplace, **Then** l'autre client voit ce mouvement de façon fluide (sans sauts/téléportations visibles).
3. **Given** un client connecté, **When** je regarde ma propre position, **Then** je vois une représentation simple de mon joueur local (capsule, cube ou repère).

---

### User Story 3 - HUD debug réseau (Priority: P3)

En tant que testeur, je veux voir un HUD minimal affichant FPS, ping, player_id et état du round pour diagnostiquer rapidement les problèmes réseau.

**Why this priority**: Le HUD debug permet de valider que le système fonctionne correctement et d'identifier rapidement les problèmes de performance ou de réseau.

**Independent Test**: Peut être testé en lançant le client et vérifiant que les informations de debug sont visibles et mises à jour en temps réel.

**Acceptance Scenarios**:

1. **Given** un client connecté à un serveur, **When** je regarde le HUD, **Then** je vois le nombre de FPS actuel affiché.
2. **Given** un client connecté à un serveur, **When** je regarde le HUD, **Then** je vois le ping/RTT actuel.
3. **Given** un client connecté à un serveur, **When** je regarde le HUD, **Then** je vois mon player_id et l'état actuel du round (countdown/playing/end).

---

### User Story 4 - Cohérence avec l'autorité serveur (Priority: P3)

En tant que développeur réseau, je veux vérifier visuellement que l'état affiché correspond à l'autorité serveur pour garantir l'intégrité du système.

**Why this priority**: La cohérence avec le serveur autoritaire est critique pour un jeu multijoueur compétitif, mais peut être validée une fois les autres éléments visuels en place.

**Independent Test**: Peut être testé en comparant les positions affichées avec les logs serveur ou en observant que les états ne divergent jamais de façon permanente.

**Acceptance Scenarios**:

1. **Given** un client connecté affichant des joueurs, **When** le serveur envoie une mise à jour de position, **Then** le client met à jour l'affichage pour correspondre à l'état serveur.
2. **Given** un client connecté, **When** une déconnexion réseau temporaire survient, **Then** l'affichage revient à l'état serveur correct après reconnexion (pas de divergence permanente).

---

### Edge Cases

- Que se passe-t-il si le client ne reçoit pas de snapshot pendant plusieurs secondes (perte de paquets)?
  - L'affichage doit conserver la dernière position connue et reprendre l'interpolation dès réception de nouveaux snapshots.
- Que se passe-t-il si un joueur se déconnecte brusquement?
  - Sa représentation visuelle doit disparaître de l'affichage des autres clients.
- Que se passe-t-il si l'arène contient des milliers de blocs?
  - Le rendu doit rester fluide (minimum 30 FPS) pour les arènes de taille raisonnable (jusqu'à 10000 blocs).
- Que se passe-t-il si le mode headless est lancé?
  - Aucun rendu graphique ne doit s'activer, les tests et load tests doivent continuer à fonctionner.

## Requirements *(mandatory)*

### Functional Requirements

#### Rendu de l'arène

- **FR-001**: Le client DOIT afficher l'arène voxel sous forme de blocs visibles dans la fenêtre.
- **FR-002**: Le rendu DOIT distinguer visuellement les sols, murs et volumes (couleurs ou nuances différentes).
- **FR-003**: L'arène affichée DOIT correspondre fidèlement à la géométrie définie dans les données d'arène.
- **FR-004**: L'arène DOIT rester statique pendant toute la session (aucune modification de blocs).

#### Visualisation des joueurs

- **FR-005**: Le client DOIT afficher une représentation visuelle du joueur local (capsule, cube ou repère visible).
- **FR-006**: Le client DOIT afficher une représentation visuelle des autres joueurs connectés.
- **FR-007**: Les mouvements des joueurs distants DOIVENT être fluides grâce à l'interpolation (pas de sauts visuels perceptibles).
- **FR-008**: L'affichage des joueurs DOIT refléter uniquement les données reçues du serveur (pas d'invention d'état).

#### Synchronisation réseau

- **FR-009**: Le client DOIT mettre à jour l'affichage à partir des snapshots reçus du serveur.
- **FR-010**: Le client DOIT pouvoir se connecter à un serveur via son adresse IP.
- **FR-011**: Le client DOIT gérer l'apparition et la disparition des joueurs (connexion/déconnexion).

#### HUD Debug

- **FR-012**: Le client DOIT afficher le nombre de FPS actuel.
- **FR-013**: Le client DOIT afficher le ping ou RTT vers le serveur.
- **FR-014**: Le client DOIT afficher le player_id du joueur local.
- **FR-015**: Le client DOIT afficher l'état du round (countdown, playing, end) et le timer si disponible.
- **FR-016**: Le HUD DOIT être lisible et stable (pas de clignotement ou de valeurs illisibles).

#### Contrôles et caméra

- **FR-017**: Le client DOIT conserver les contrôles FPS existants (WASD + souris pour le mouvement et la rotation de caméra).
- **FR-018**: La caméra DOIT permettre d'explorer l'arène librement pour valider le rendu.

#### Modes d'exécution

- **FR-019**: Le mode windowed DOIT être le mode par défaut pour la visualisation.
- **FR-020**: Le mode headless DOIT continuer à fonctionner pour les tests et load tests (sans activer de rendu graphique).
- **FR-021**: Les tests existants DOIVENT continuer à passer sans modification.

### Key Entities

- **Arène (Arena)**: Représente l'environnement de jeu composé de blocs voxels. Contient la géométrie statique (positions et types de blocs).
- **Joueur (Player)**: Représente un participant dans la partie. Possède une position 3D, une orientation, et un identifiant unique (player_id).
- **Snapshot**: État du monde à un instant T envoyé par le serveur. Contient les positions de tous les joueurs et l'état du round.
- **Round**: Phase de jeu avec un état (countdown, playing, end) et optionnellement un timer associé.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Un utilisateur peut lancer le serveur puis le client et voir l'arène voxel rendue en moins de 10 secondes après démarrage.
- **SC-002**: Deux clients connectés au même serveur voient chacun les deux joueurs dans l'arène.
- **SC-003**: Le mouvement des joueurs distants apparaît fluide (sans sauts visibles lors d'un déplacement continu à vitesse normale).
- **SC-004**: Le HUD affiche simultanément FPS, ping/RTT, player_id et état du round avec des valeurs mises à jour.
- **SC-005**: Le client maintient au minimum 30 FPS sur une arène de test standard pendant l'exploration.
- **SC-006**: Le mode headless démarre sans erreur et les scripts de load test s'exécutent avec succès.
- **SC-007**: 100% des tests automatisés existants passent après l'implémentation de cette feature.
- **SC-008**: Le ping affiché dans le HUD correspond à la latence réseau réelle (variation de moins de 50ms par rapport à la mesure système).

## Constraints

- Aucune fonctionnalité produit ajoutée (pas de server browser, pas d'authentification, pas de comptes).
- Pas d'intégration web ni de navigateur embarqué (pas de CEF).
- Arènes prédéfinies uniquement (pas de génération procédurale).
- Arène statique (pas de modification de blocs place/remove).
- Pas de persistance du monde.
- Pas de textures avancées ni d'éclairage complexe.
- Pas d'inventaire, crafting, mobs ou mods.

## Assumptions

- Le serveur existant fournit déjà les snapshots avec les positions des joueurs et l'état du round.
- Le format de données de l'arène est déjà défini et accessible au client.
- Le système de connexion par IP existe et fonctionne.
- Les contrôles FPS (WASD + souris) sont déjà implémentés dans le client.
- Une arène de test existe pour valider le rendu.

## Out of Scope

- CEF / UI web
- Textures réalistes ou lighting avancé
- Blocs dynamiques (place/remove)
- Inventaire, crafting, mobs
- Système de mods
- Server browser centralisé
- Comptes joueurs et authentification
- Effets sonores
- Système de particules
