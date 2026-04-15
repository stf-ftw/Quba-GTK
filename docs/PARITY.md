# Quba GTK Parity Checklist

This checklist tracks the native rewrite against the original Electron/Vue Quba viewer.

## Implemented

- Native GTK4/libadwaita shell
- Open files from command line and desktop activation
- Desktop entry for `Open With`
- Open XML invoices directly
- Open hybrid PDFs and extract embedded XML
- Reuse legacy XSLT/Saxon rendering assets
- Render structured invoice HTML inside WebKit
- Show raw XML/details in a native source viewer
- Multiple tabs
- English and Romanian UI strings
- Drag and drop files into the window
- About button
- Examples website button

## Partial

- PDF support
  The app extracts embedded XML and can hand the PDF off to the default PDF app, but it does not yet embed a full in-app PDF viewer like the original PDF.js split view.
- Language support
  Only English and Romanian are implemented in the GTK shell. The original shipped more translations.

## Missing

- In-app PDF rendering and split PDF/XML layout parity
- Search parity across rendered invoice HTML and PDF
- Print actions for PDF and XML
- Validation flow and login dialog
- Show IDs toggle passed through the XSLT UI
- Manual/help content window
- Update workflow
- Settings persistence for language and viewer preferences
