#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const SaxonJS = require("saxon-js");
const parser = require("fast-xml-parser");
const pdfjsLib = require("pdfjs-dist/legacy/build/pdf");

const projectRoot = path.resolve(__dirname, "..");
const legacyRoot = path.join(projectRoot, "quba-viewer-1.5.0");
const xsltRoot = path.join(legacyRoot, "app", "xslt");
const translationsPath = path.join(legacyRoot, "src", "translation", "en.json");

function rootKey(xml) {
  const parsed = parser.parse(xml);
  const keys = Object.keys(parsed);
  return keys[0] || "";
}

function stylesheetForXml(xml) {
  const key = rootKey(xml);
  if (key.includes("CrossIndustryInvoice")) {
    return { stylesheet: "cii-xr.sef.json", mode: "default" };
  }
  if (key.includes("SCRDMCCBDACIOMessageStructure")) {
    return { stylesheet: "cio-xr.sef.json", mode: "default" };
  }
  if (key.includes("Invoice")) {
    return { stylesheet: "ubl-xr.sef.json", mode: "default" };
  }
  if (key.includes("CreditNote")) {
    return { stylesheet: "ubl-creditnote-xr.sef.json", mode: "default" };
  }
  if (key.includes("CrossIndustryDocument")) {
    return { stylesheet: "ZUGFeRD_1p0_c1p0_s1p0.sef.json", mode: "zf1" };
  }
  throw new Error(`Unsupported XML document root: ${key}`);
}

function isOrderDocument(xml) {
  return /<[^>]*TypeCode[^>]*>\s*(220|231)\s*<\/[^>]*TypeCode>/i.test(xml);
}

async function renderXml(xml, title) {
  const kind = stylesheetForXml(xml);
  const isOrder = isOrderDocument(xml);
  if (kind.mode === "zf1") {
    const output = await SaxonJS.transform(
      {
        stylesheetFileName: path.join(xsltRoot, kind.stylesheet),
        sourceText: xml,
        destination: "serialized",
      },
      "async"
    );

    return {
      title,
      html: output.principalResult,
      xml,
      raw_kind: "zf1",
      warnings: [],
    };
  }

  const intermediate = await SaxonJS.transform(
    {
      stylesheetFileName: path.join(xsltRoot, kind.stylesheet),
      sourceText: xml,
      destination: "serialized",
    },
    "async"
  );

  const translations = JSON.parse(fs.readFileSync(translationsPath, "utf8"));
  if (isOrder) {
    translations.bt1 = "Order number";
    translations.bt2 = "Order date";
    translations.bt3 = "Order type";
    translations.bg22 = "Order totals";
    translations.bt25 = "Referenced order number";
    translations.bt26 = "Referenced order date";
    translations.details = "Order details";
  }

  const rendered = await SaxonJS.transform(
    {
      stylesheetFileName: path.join(xsltRoot, "xrechnung-html.uni.sef.json"),
      sourceText: intermediate.principalResult,
      destination: "serialized",
      stylesheetParams: {
        isOrder,
        showIds: false,
        "Q{}i18n": translations,
      },
    },
    "async"
  );

  return {
    title,
    html: rendered.principalResult,
    xml,
    raw_kind: kind.stylesheet,
    warnings: [],
  };
}

async function renderXmlFile(filePath) {
  const xml = fs.readFileSync(filePath, "utf8");
  return renderXml(xml, path.basename(filePath));
}

async function renderPdfFile(filePath) {
  const loadingTask = pdfjsLib.getDocument({
    url: filePath,
    disableWorker: true,
  });
  const pdf = await loadingTask.promise;
  const attachments = (await pdf.getAttachments()) || {};

  let xml = null;
  const candidates = Object.values(attachments).filter(Boolean);
  for (const entry of candidates) {
    const filename = (entry.filename || "").toLowerCase();
    if (
      filename === "factur-x.xml" ||
      filename === "zugferd-invoice.xml" ||
      filename === "xrechnung.xml" ||
      filename === "order-x.xml"
    ) {
      xml = new TextDecoder().decode(entry.content);
      break;
    }
  }

  if (!xml) {
    return {
      title: path.basename(filePath),
      html: null,
      xml: null,
      raw_kind: "pdf",
      warnings: ["No embedded XML attachment was found in this PDF."],
    };
  }

  const rendered = await renderXml(xml, `${path.basename(filePath)} (embedded XML)`);
  return {
    ...rendered,
    warnings: rendered.warnings || [],
  };
}

async function main() {
  const [, , command, targetPath] = process.argv;
  if (!command || !targetPath) {
    throw new Error("Usage: quba-render-helper.cjs <render-xml|render-pdf> <path>");
  }

  let response;
  if (command === "render-xml") {
    response = await renderXmlFile(targetPath);
  } else if (command === "render-pdf") {
    response = await renderPdfFile(targetPath);
  } else {
    throw new Error(`Unknown command: ${command}`);
  }

  process.stdout.write(JSON.stringify(response));
}

main().catch((error) => {
  console.error(error.stack || error.message || String(error));
  process.exit(1);
});
