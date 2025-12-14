# Feature Specification: Timestamp on Selection Rectangle

**Feature Branch**: `1-timestamp-overlay`
**Created**: 2024-12-14
**Status**: Draft
**Input**: User description: "affiche l'horodatage directement sur le rectangle de capture d'écran et retire la petite miniature de la fenêtre."

## Clarifications

### Session 2024-12-14

- Q: Où doit apparaître la bande (banner) de l'horodatage par rapport au rectangle de sélection ? → A: À l'extérieur du rectangle (la sélection reste intacte, la bande s'ajoute visuellement)
- Q: Comment doit se comporter l'aperçu de l'horodatage lors d'une capture plein écran ? → A: Forcer l'affichage de l'interface de sélection même pour le plein écran (rectangle couvrant tout l'écran avec aperçu de l'horodatage)
- Q: Quand doit s'afficher le dialogue filename-dialog par rapport à la sélection ? → A: Le dialogue doit s'afficher en même temps que la fenêtre de sélection (pas après confirmation) pour permettre la modification des options d'horodatage en temps réel sur l'aperçu
- Q: Quand la sélection est-elle considérée comme confirmée ? → A: La sélection reste modifiable indéfiniment jusqu'au clic sur "Sauvegarder" dans le dialogue (pas besoin d'appuyer sur Entrée)
- Q: Comment la fenêtre de dialogue doit-elle rester visible ? → A: Le dialogue est "always on top" (toujours au-dessus de toutes les fenêtres, y compris la sélection)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Live Timestamp Preview During Selection (Priority: P1)

As a user performing a screen capture, I want to see the timestamp directly on my selection rectangle in real-time so that I can preview exactly how my final screenshot will look before confirming the capture.

**Why this priority**: This is the core feature request - users need immediate visual feedback of the timestamp placement while making their selection. This represents the primary user value.

**Independent Test**: Can be fully tested by starting a screen capture, drawing a selection rectangle, and verifying the timestamp appears on the selection overlay with the configured style (position, color, font size).

**Acceptance Scenarios**:

1. **Given** timestamp is enabled in settings, **When** user draws a selection rectangle during capture, **Then** the timestamp appears on the selection rectangle preview in the configured position (top or bottom)
2. **Given** timestamp is enabled with "overlay" type, **When** user draws a selection rectangle, **Then** the timestamp appears directly over the image area
3. **Given** timestamp is enabled with "banner" type, **When** user draws a selection rectangle, **Then** a banner area is shown at the configured position with the timestamp
4. **Given** timestamp styling options are configured (font size, color, alignment), **When** user views the selection rectangle, **Then** the timestamp reflects all configured styles
5. **Given** user modifies the selection rectangle (resize or move), **When** the selection changes, **Then** the timestamp preview updates accordingly
6. **Given** user initiates fullscreen capture (Win+Shift+PrintScreen), **When** the selection interface opens, **Then** a fullscreen selection rectangle is displayed with the timestamp preview visible
7. **Given** user opens selection window, **When** the window appears, **Then** the options dialog (filename-dialog) opens simultaneously alongside the selection window
8. **Given** both windows are open, **When** user modifies timestamp options in the dialog, **Then** the timestamp preview on the selection rectangle updates in real-time
9. **Given** both windows are open, **When** user clicks on the selection window, **Then** the dialog remains visible (always on top) and user can modify the selection
10. **Given** user has drawn a selection, **When** user clicks "Save" in the dialog, **Then** the screenshot is captured with the current selection bounds (no Enter key required)

---

### User Story 2 - Remove Preview Thumbnail from Save Dialog (Priority: P2)

As a user saving a screenshot, I want a simplified save dialog without the preview thumbnail so that the interface is cleaner and faster to use.

**Why this priority**: This is a simplification feature that removes visual clutter. It depends on User Story 1 being implemented since users will now see the preview during selection instead.

**Independent Test**: Can be fully tested by completing a capture and verifying the save dialog appears without the preview canvas, with reduced window height.

**Acceptance Scenarios**:

1. **Given** user completes a selection and confirms capture, **When** the save dialog opens, **Then** the preview thumbnail canvas is not displayed
2. **Given** the preview thumbnail is removed, **When** the save dialog opens, **Then** the window height is reduced appropriately to fit the remaining content
3. **Given** the preview is removed from the dialog, **When** user changes timestamp options in the save dialog, **Then** there is no visual preview update (options are saved for next capture)

---

### Edge Cases

- What happens when timestamp is disabled? The selection rectangle displays without any timestamp overlay.
- What happens with very large font sizes? The timestamp may extend beyond the selection rectangle boundaries; it should be clipped to the selection area.
- What happens with very small selection rectangles? The timestamp should still be visible but may be truncated or scaled down to fit.
- What happens when the selection is being resized? The timestamp position should update smoothly as the rectangle dimensions change.
- What happens for fullscreen capture? The selection interface is displayed with a rectangle covering the entire screen, showing the timestamp preview. User clicks "Save" in the dialog to complete capture.
- What happens when user closes filename-dialog without saving? Both windows close; the capture is cancelled.
- What happens when user modifies options in filename-dialog? Changes are immediately reflected on the selection rectangle timestamp preview via inter-window communication.
- What happens when user clicks on the selection window while dialog is open? The dialog remains visible (always on top) and the user can continue modifying the selection.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display timestamp on the selection rectangle overlay when timestamp is enabled in settings
- **FR-002**: System MUST update the timestamp preview in real-time as the user draws or modifies the selection rectangle
- **FR-003**: System MUST apply all configured timestamp styling (type, position, font size, text color, text alignment, bold, italic, underline) to the preview
- **FR-004**: System MUST display the current date and time in the French locale format (as currently implemented)
- **FR-005**: System MUST render banner-style timestamps by showing a colored banner area outside the selection rectangle at the configured position (top or bottom), preserving the full captured area
- **FR-006**: System MUST render overlay-style timestamps by displaying the text directly over the image area
- **FR-007**: System MUST remove the preview canvas element from the save dialog (filename-dialog.html)
- **FR-008**: System MUST adjust the save dialog window height after removing the preview thumbnail
- **FR-009**: System MUST preserve all existing timestamp configuration options in the save dialog for modifying settings
- **FR-010**: System MUST open the filename-dialog window simultaneously with the selection window (not after confirmation)
- **FR-011**: System MUST broadcast timestamp option changes from filename-dialog to the selection window in real-time
- **FR-012**: System MUST update the timestamp preview on the selection rectangle when options are modified in the dialog
- **FR-013**: System MUST NOT require pressing Enter to confirm the selection; the selection remains modifiable until user clicks "Save" in the dialog
- **FR-014**: System MUST set the filename-dialog window as "always on top" so it remains visible above the selection window at all times

### Key Entities *(include if feature involves data)*

- **Selection Rectangle**: The user-drawn area defining the screenshot boundaries, now including timestamp overlay rendering
- **Timestamp Options**: Configuration object containing enabled state, type, position, fontSize, textColor, textAlign, bold, italic, underline properties

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can see the timestamp preview on the selection rectangle immediately upon drawing (within the same frame update)
- **SC-002**: The timestamp preview matches 100% with the final saved screenshot appearance when using the same options
- **SC-003**: Save dialog opens successfully without the preview thumbnail and displays at a reduced height
- **SC-004**: All timestamp styling options (9 options total) are correctly reflected in the selection rectangle preview
- **SC-005**: Users complete the capture workflow in fewer steps since they no longer need to preview changes in the save dialog
