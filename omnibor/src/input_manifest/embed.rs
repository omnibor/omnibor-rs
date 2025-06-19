use {
    crate::{
        artifact_id::ArtifactId,
        error::{EmbeddingError, InputManifestError},
        hash_algorithm::HashAlgorithm,
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    bindet::FileType as BinaryType,
    std::{fmt::Debug, fs::OpenOptions, io::Write as _, ops::Not, path::Path},
};

const EMBED_KEY: &str = "OmniBOR-Input-Manifest";

/// Embed the manifest's [`ArtifactId`] into the target file.
pub(crate) fn embed_manifest_in_target<H: HashAlgorithm>(
    target_path: &Path,
    manifest_aid: ArtifactId<H>,
) -> Result<Result<(), EmbeddingError>, InputManifestError> {
    match TargetType::infer(target_path)? {
        TargetType::KnownBinaryType(binary_type) => Ok(Err(
            EmbeddingError::UnsupportedBinaryFormat(binary_type.name().clone_as_boxstr()),
        )),
        TargetType::KnownTextType(TextType::PrefixComments { prefix }) => {
            embed_in_text_file_with_prefix_comment(target_path, manifest_aid, prefix)
        }
        TargetType::KnownTextType(TextType::WrappedComments { prefix, suffix }) => {
            embed_in_text_file_with_wrapped_comment(target_path, manifest_aid, prefix, suffix)
        }
        TargetType::KnownTextType(TextType::NoComments(name)) => Ok(Err(
            EmbeddingError::FormatDoesntSupportEmbedding(name.clone_as_boxstr()),
        )),
        TargetType::KnownTextType(TextType::UnknownComments(name)) => Ok(Err(
            EmbeddingError::UnknownEmbeddingSupport(name.clone_as_boxstr()),
        )),
        TargetType::Unknown => Ok(Err(EmbeddingError::UnknownEmbeddingTarget)),
    }
}

fn embed_in_text_file_with_prefix_comment<H: HashAlgorithm>(
    path: &Path,
    manifest_aid: ArtifactId<H>,
    prefix: &str,
) -> Result<Result<(), EmbeddingError>, InputManifestError> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|source| InputManifestError::FailedTargetArtifactRead(Box::new(source)))?;

    let result =
        writeln!(&mut file, "{} {}: {}", prefix, EMBED_KEY, manifest_aid).map_err(|source| {
            EmbeddingError::CantEmbedInTarget(path.clone_as_boxstr(), Box::new(source))
        });

    Ok(result)
}

fn embed_in_text_file_with_wrapped_comment<H: HashAlgorithm>(
    path: &Path,
    manifest_aid: ArtifactId<H>,
    prefix: &str,
    suffix: &str,
) -> Result<Result<(), EmbeddingError>, InputManifestError> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|source| InputManifestError::FailedTargetArtifactRead(Box::new(source)))?;

    let result = writeln!(
        &mut file,
        "{} {}: {} {}",
        prefix, EMBED_KEY, manifest_aid, suffix
    )
    .map_err(|source| EmbeddingError::CantEmbedInTarget(path.clone_as_boxstr(), Box::new(source)));

    Ok(result)
}

trait BinaryTypeName {
    fn name(&self) -> &'static str;
}

impl BinaryTypeName for BinaryType {
    fn name(&self) -> &'static str {
        match self {
            BinaryType::Zip => "zip",
            BinaryType::Rar5 => "rar5",
            BinaryType::Rar => "rar",
            BinaryType::Tar => "tar",
            BinaryType::Lzma => "lzma",
            BinaryType::Xz => "xz",
            BinaryType::Zst => "zst",
            BinaryType::Png => "png",
            BinaryType::Jpg => "jpg",
            BinaryType::_7z => "7z",
            BinaryType::Opus => "opus",
            BinaryType::Vorbis => "vorbis",
            BinaryType::Mp3 => "mp3",
            BinaryType::Webp => "webp",
            BinaryType::Flac => "flac",
            BinaryType::Matroska => "matroska",
            BinaryType::Wasm => "wasm",
            BinaryType::Class => "class",
            BinaryType::Tasty => "tasty",
            BinaryType::Mach => "mach",
            BinaryType::Elf => "elf",
            BinaryType::Wav => "wav",
            BinaryType::Avi => "avi",
            BinaryType::Aiff => "aiff",
            BinaryType::Tiff => "tiff",
            BinaryType::Sqlite3 => "sqlite3",
            BinaryType::Ico => "ico",
            BinaryType::Dalvik => "dalvik",
            BinaryType::Pdf => "pdf",
            BinaryType::DosMzExecutable => "dos-mz-executable",
            BinaryType::DosZmExecutable => "dos-zm-executable",
            BinaryType::Xcf => "xcf",
            BinaryType::Gif => "gif",
            BinaryType::Bmp => "bmp",
            BinaryType::Iso => "iso",
            BinaryType::Gpg => "gpg",
            BinaryType::ArmoredGpg => "armored-gpg",
            BinaryType::Swf => "swf",
            BinaryType::Swc => "swc",
            _ => "(unknown)",
        }
    }
}

#[derive(Debug)]
enum TargetType {
    KnownBinaryType(BinaryType),
    KnownTextType(TextType),
    Unknown,
}

impl TargetType {
    fn infer(path: &Path) -> Result<Self, InputManifestError> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|source| InputManifestError::FailedTargetArtifactRead(Box::new(source)))?;

        // Try binary detection first, as it's faster.
        if let Ok(Some(file_type_matches)) = bindet::detect(file) {
            if file_type_matches.likely_to_be.is_empty().not() {
                // For now, just use the first detected match.
                return Ok(TargetType::KnownBinaryType(
                    file_type_matches.likely_to_be[0],
                ));
            }

            if file_type_matches.all_matches.is_empty().not() {
                // For now, just use the first detected match.
                return Ok(TargetType::KnownBinaryType(
                    file_type_matches.all_matches[0].file_type,
                ));
            }
        }

        // Fall back to text detection.
        let target_type = KnownProgLang::infer(path)
            .as_ref()
            .map(KnownProgLang::text_type)
            .map(TargetType::KnownTextType)
            .unwrap_or(TargetType::Unknown);

        Ok(target_type)
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum TextType {
    /// The file format supports "prefix" comments with just a prefix.
    PrefixComments { prefix: &'static str },
    /// The file format supports "wrapped" comments with a prefix and suffix.
    WrappedComments {
        prefix: &'static str,
        suffix: &'static str,
    },
    /// The file format does not support comments.
    NoComments(String),
    /// The type of comments supported by the file is unknown.
    UnknownComments(String),
}

impl KnownProgLang {
    /// Try to infer a known programming language for the file at the path.
    fn infer(path: &Path) -> Option<Self> {
        let detection = match hyperpolyglot::detect(path) {
            Ok(Some(detection)) => detection,
            _ => return None,
        };

        let Ok(language) = detection.language().parse::<KnownProgLang>() else {
            return None;
        };

        Some(language)
    }

    fn text_type(&self) -> TextType {
        use KnownProgLang::*;
        use TextType::*;

        match self {
            Python | Perl | R | Ruby | Julia => PrefixComments { prefix: "#" },
            Cpp | C | Java | CSharp | JavaScript | Go | Pascal | Php | Rust | Kotlin | Swift
            | Dart | TypeScript | Scala | ObjectiveC => PrefixComments { prefix: "//" },
            VisualBasicDotNet => PrefixComments { prefix: "'" },
            Fortran => PrefixComments { prefix: "!" },
            Ada | Sql | Haskell | Lua => PrefixComments { prefix: "--" },
            Matlab | Prolog => PrefixComments { prefix: "%" },
            Assembly | CommonLisp | Scheme => PrefixComments { prefix: ";" },
            Cobol => PrefixComments { prefix: "*" },
            Css => WrappedComments {
                prefix: "/*",
                suffix: "*/",
            },
            Json => NoComments(self.to_string()),
            _ => UnknownComments(self.to_string()),
        }
    }
}

/// Known programming languages, to tie into the output of hyperpolyglot.
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumString, strum::Display)]
#[strum(use_phf)]
enum KnownProgLang {
    #[strum(to_string = "F*")]
    FStar,
    #[strum(to_string = "Git Attributes")]
    GitAttributes,
    #[strum(to_string = "IRC Log")]
    IrcLog,
    #[strum(to_string = "ECLiPSe")]
    Eclipse,
    #[strum(to_string = "Blade")]
    Blade,
    #[strum(to_string = "Pure Data")]
    PureData,
    #[strum(to_string = "Grammatical Framework")]
    GrammaticalFramework,
    #[strum(to_string = "Brightscript")]
    Brightscript,
    #[strum(to_string = "Go")]
    Go,
    #[strum(to_string = "BitBake")]
    BitBake,
    #[strum(to_string = "COBOL")]
    Cobol,
    #[strum(to_string = "Dhall")]
    Dhall,
    #[strum(to_string = "OpenCL")]
    OpenCl,
    #[strum(to_string = "Pod 6")]
    Pod6,
    #[strum(to_string = "Scheme")]
    Scheme,
    #[strum(to_string = "DIGITAL Command Language")]
    DigitalCommandLanguage,
    #[strum(to_string = "Diff")]
    Diff,
    #[strum(to_string = "Motorola 68K Assembly")]
    Motorola68KAssembly,
    #[strum(to_string = "Io")]
    Io,
    #[strum(to_string = "SRecode Template")]
    SRecodeTemplate,
    #[strum(to_string = "SCSS")]
    Scss,
    #[strum(to_string = "AMPL")]
    Ampl,
    #[strum(to_string = "Regular Expression")]
    RegularExpression,
    #[strum(to_string = "CoNLL-U")]
    CoNllU,
    #[strum(to_string = "nanorc")]
    NanoRc,
    #[strum(to_string = "Slice")]
    Slice,
    #[strum(to_string = "OpenSCAD")]
    OpenScad,
    #[strum(to_string = "JSX")]
    Jsx,
    #[strum(to_string = "YANG")]
    Yang,
    #[strum(to_string = "F#")]
    FSharp,
    #[strum(to_string = "X10")]
    X10,
    #[strum(to_string = "Common Workflow Language")]
    CommonWorkflowLanguage,
    #[strum(to_string = "Stan")]
    Stan,
    #[strum(to_string = "Befunge")]
    Befunge,
    #[strum(to_string = "Gentoo Ebuild")]
    GentooEbuild,
    #[strum(to_string = "Altium Designer")]
    AltiumDesigner,
    #[strum(to_string = "Alloy")]
    Alloy,
    #[strum(to_string = "Sage")]
    Sage,
    #[strum(to_string = "Object Data Instance Notation")]
    ObjectDataInstanceNotation,
    #[strum(to_string = "ZenScript")]
    ZenScript,
    #[strum(to_string = "AntBuildSystem")]
    AntBuildSystem,
    #[strum(to_string = "Pep8")]
    Pep8,
    #[strum(to_string = "BibTeX")]
    BibTeX,
    #[strum(to_string = "JSON")]
    Json,
    #[strum(to_string = "Maven POM")]
    MavenPom,
    #[strum(to_string = "Smarty")]
    Smarty,
    #[strum(to_string = "Wavefront Object")]
    WavefrontObject,
    #[strum(to_string = "C++")]
    Cpp,
    #[strum(to_string = "Ignore List")]
    IgnoreList,
    #[strum(to_string = "DirectX 3D File")]
    DirectX3DFile,
    #[strum(to_string = "Apex")]
    Apex,
    #[strum(to_string = "LLVM")]
    Llvm,
    #[strum(to_string = "UnrealScript")]
    UnrealScript,
    #[strum(to_string = "TLA")]
    Tla,
    #[strum(to_string = "HCL")]
    Hcl,
    #[strum(to_string = "Edje Data Collection")]
    EdjeDataCollection,
    #[strum(to_string = "Fancy")]
    Fancy,
    #[strum(to_string = "KRL")]
    Krl,
    #[strum(to_string = "ECL")]
    Ecl,
    #[strum(to_string = "Roff Manpage")]
    RoffManpage,
    #[strum(to_string = "Objective-C++")]
    ObjectiveCpp,
    #[strum(to_string = "Turing")]
    Turing,
    #[strum(to_string = "ShaderLab")]
    ShaderLab,
    #[strum(to_string = "Makefile")]
    Makefile,
    #[strum(to_string = "Factor")]
    Factor,
    #[strum(to_string = "CWeb")]
    CWeb,
    #[strum(to_string = "LSL")]
    Lsl,
    #[strum(to_string = "Pony")]
    Pony,
    #[strum(to_string = "Gherkin")]
    Gherkin,
    #[strum(to_string = "Awk")]
    Awk,
    #[strum(to_string = "Cycript")]
    Cycript,
    #[strum(to_string = "Jolie")]
    Jolie,
    #[strum(to_string = "Ballerina")]
    Ballerina,
    #[strum(to_string = "Starlark")]
    Starlark,
    #[strum(to_string = "Gettext Catalog")]
    GettextCatalog,
    #[strum(to_string = "Lolcode")]
    Lolcode,
    #[strum(to_string = "Oz")]
    Oz,
    #[strum(to_string = "PlantUML")]
    PlantUml,
    #[strum(to_string = "SMT")]
    Smt,
    #[strum(to_string = "V")]
    V,
    #[strum(to_string = "Nix")]
    Nix,
    #[strum(to_string = "Zeek")]
    Zeek,
    #[strum(to_string = "Jison Lex")]
    JisonLex,
    #[strum(to_string = "cURL Config")]
    CurlConfig,
    #[strum(to_string = "KiCad Layout")]
    KiCadLayout,
    #[strum(to_string = "HTML+EEX")]
    HtmlPlusEex,
    #[strum(to_string = "Mask")]
    Mask,
    #[strum(to_string = "Isabelle")]
    Isabelle,
    #[strum(to_string = "LoomScript")]
    LoomScript,
    #[strum(to_string = "Raku")]
    Raku,
    #[strum(to_string = "VBA")]
    Vba,
    #[strum(to_string = "ASN.1")]
    Asn1,
    #[strum(to_string = "Brainfuck")]
    Brainfuck,
    #[strum(to_string = "DTrace")]
    DTrace,
    #[strum(to_string = "Elm")]
    Elm,
    #[strum(to_string = "Crystal")]
    Crystal,
    #[strum(to_string = "Fortran")]
    Fortran,
    #[strum(to_string = "ChucK")]
    ChucK,
    #[strum(to_string = "HyPhy")]
    HyPhy,
    #[strum(to_string = "Genie")]
    Genie,
    #[strum(to_string = "MQL5")]
    Mql5,
    #[strum(to_string = "PLSQL")]
    PlSql,
    #[strum(to_string = "Jasmin")]
    Jasmin,
    #[strum(to_string = "desktop")]
    Desktop,
    #[strum(to_string = "Self")]
    SelfLang,
    #[strum(to_string = "Swift")]
    Swift,
    #[strum(to_string = "mIRC Script")]
    MIrcScript,
    #[strum(to_string = "Smali")]
    Smali,
    #[strum(to_string = "eC")]
    Ec,
    #[strum(to_string = "JavaServerPages")]
    JavaServerPages,
    #[strum(to_string = "Lean")]
    Lean,
    #[strum(to_string = "Oxygene")]
    Oxygene,
    #[strum(to_string = "STON")]
    Ston,
    #[strum(to_string = "Inno Setup")]
    InnoSetup,
    #[strum(to_string = "ASP")]
    Asp,
    #[strum(to_string = "Gosu")]
    Gosu,
    #[strum(to_string = "Gradle")]
    Gradle,
    #[strum(to_string = "Objective-J")]
    ObjectiveJ,
    #[strum(to_string = "Volt")]
    Volt,
    #[strum(to_string = "XC")]
    Xc,
    #[strum(to_string = "Golo")]
    Golo,
    #[strum(to_string = "EditorConfig")]
    EditorConfig,
    #[strum(to_string = "G-code")]
    GCode,
    #[strum(to_string = "Mirah")]
    Mirah,
    #[strum(to_string = "Python console")]
    PythonConsole,
    #[strum(to_string = "Inform 7")]
    Inform7,
    #[strum(to_string = "VHDL")]
    Vhdl,
    #[strum(to_string = "M4")]
    M4,
    #[strum(to_string = "q")]
    Q,
    #[strum(to_string = "ColdFusion")]
    ColdFusion,
    #[strum(to_string = "ANTLR")]
    Antlr,
    #[strum(to_string = "Ren'Py")]
    RenPy,
    #[strum(to_string = "EML")]
    Eml,
    #[strum(to_string = "TSX")]
    Tsx,
    #[strum(to_string = "NumPy")]
    NumPy,
    #[strum(to_string = "nesC")]
    NesC,
    #[strum(to_string = "Modula-3")]
    Modula3,
    #[strum(to_string = "NewLisp")]
    NewLisp,
    #[strum(to_string = "Pug")]
    Pug,
    #[strum(to_string = "Grace")]
    Grace,
    #[strum(to_string = "Idris")]
    Idris,
    #[strum(to_string = "YASnippet")]
    YaSnippet,
    #[strum(to_string = "Parrot Internal Representation")]
    ParrotInternalRepresentation,
    #[strum(to_string = "SaltStack")]
    SaltStackm,
    #[strum(to_string = "REXX")]
    Rexx,
    #[strum(to_string = "Cloud Firestore Security Rules")]
    CloudFirestoreSecurityRules,
    #[strum(to_string = "INI")]
    Ini,
    #[strum(to_string = "MATLAB")]
    Matlab,
    #[strum(to_string = "Svelte")]
    Svelte,
    #[strum(to_string = "Text")]
    Text,
    #[strum(to_string = "Arc")]
    Arc,
    #[strum(to_string = "Xojo")]
    Xojo,
    #[strum(to_string = "HTML+Django")]
    HtmlPlusDjango,
    #[strum(to_string = "AngelScript")]
    AngelScript,
    #[strum(to_string = "Slash")]
    Slash,
    #[strum(to_string = "Zephir")]
    Zephir,
    #[strum(to_string = "Vim Snippet")]
    VimSmippet,
    #[strum(to_string = "SuperCollider")]
    SuperCollider,
    #[strum(to_string = "Modula-2")]
    Modula2,
    #[strum(to_string = "Web Ontology Language")]
    WebOntologyLanguage,
    #[strum(to_string = "Handlebars")]
    Handlebars,
    #[strum(to_string = "COLLADA")]
    Collada,
    #[strum(to_string = "HTML+Razor")]
    HtmlPlusRazor,
    #[strum(to_string = "SPARQL")]
    Sparql,
    #[strum(to_string = "Texinfo")]
    Texinfo,
    #[strum(to_string = "Stata")]
    Stata,
    #[strum(to_string = "Opa")]
    Opa,
    #[strum(to_string = "Ring")]
    Ring,
    #[strum(to_string = "WebAssembly")]
    WebAssembly,
    #[strum(to_string = "ABAP")]
    Abap,
    #[strum(to_string = "FLUX")]
    Flex,
    #[strum(to_string = "Nim")]
    Nim,
    #[strum(to_string = "GAMS")]
    Gams,
    #[strum(to_string = "SAS")]
    Sas,
    #[strum(to_string = "SQL")]
    Sql,
    #[strum(to_string = "CSON")]
    Cson,
    #[strum(to_string = "Harbour")]
    Harbour,
    #[strum(to_string = "RHTML")]
    RHtml,
    #[strum(to_string = "Nemerle")]
    Nemerle,
    #[strum(to_string = "IDL")]
    Idl,
    #[strum(to_string = "JSON with Comments")]
    JsonWithComments,
    #[strum(to_string = "ActionScript")]
    ActionScript,
    #[strum(to_string = "MediaWiki")]
    MediaWiki,
    #[strum(to_string = "Nginx")]
    Nginx,
    #[strum(to_string = "Less")]
    Less,
    #[strum(to_string = "Forth")]
    Forth,
    #[strum(to_string = "OpenRC runscript")]
    OpenRcRunscript,
    #[strum(to_string = "APL")]
    Apl,
    #[strum(to_string = "Python")]
    Python,
    #[strum(to_string = "Scaml")]
    Scaml,
    #[strum(to_string = "Verilog")]
    Verilog,
    #[strum(to_string = "QML")]
    Qml,
    #[strum(to_string = "Assembly")]
    Assembly,
    #[strum(to_string = "Marko")]
    Marko,
    #[strum(to_string = "D-ObjDump")]
    DObjDump,
    #[strum(to_string = "P4")]
    P4,
    #[strum(to_string = "sed")]
    Sed,
    #[strum(to_string = "Rouge")]
    Rouge,
    #[strum(to_string = "Pan")]
    Pan,
    #[strum(to_string = "DNS Zone")]
    DnsZone,
    #[strum(to_string = "RPM Spec")]
    RpmSpec,
    #[strum(to_string = "JavaScript")]
    JavaScript,
    #[strum(to_string = "SystemVerilog")]
    SystemVerilog,
    #[strum(to_string = "Wollok")]
    Wollok,
    #[strum(to_string = "R")]
    R,
    #[strum(to_string = "Latte")]
    Latte,
    #[strum(to_string = "Mathematica")]
    Mathematica,
    #[strum(to_string = "EBNF")]
    Ebnf,
    #[strum(to_string = "Rebol")]
    Rebol,
    #[strum(to_string = "SWIG")]
    Swig,
    #[strum(to_string = "JFlex")]
    JFlex,
    #[strum(to_string = "FIGlet Font")]
    FigletFont,
    #[strum(to_string = "dircolors")]
    Dircolors,
    #[strum(to_string = "Shen")]
    Shen,
    #[strum(to_string = "Scala")]
    Scala,
    #[strum(to_string = "AGS Script")]
    AgsScript,
    #[strum(to_string = "LilyPond")]
    LilyPond,
    #[strum(to_string = "Standard ML")]
    StandardMl,
    #[strum(to_string = "Bluespec")]
    Bluespec,
    #[strum(to_string = "Unity3D Asset")]
    Unity3DAsset,
    #[strum(to_string = "Markdown")]
    Markdown,
    #[strum(to_string = "API Blueprint")]
    ApiBlueprint,
    #[strum(to_string = "Logos")]
    Logos,
    #[strum(to_string = "Readline Config")]
    ReadlineConfig,
    #[strum(to_string = "Cool")]
    Cool,
    #[strum(to_string = "REALbasic")]
    RealBasic,
    #[strum(to_string = "CodeQL")]
    CodeQl,
    #[strum(to_string = "WebIDL")]
    WebIdl,
    #[strum(to_string = "CLIPS")]
    Clips,
    #[strum(to_string = "GAP")]
    Gap,
    #[strum(to_string = "Nu")]
    Nu,
    #[strum(to_string = "Game Maker Language")]
    GameMakerLanguage,
    #[strum(to_string = "PostScript")]
    PostScript,
    #[strum(to_string = "HAProxy")]
    HaProxy,
    #[strum(to_string = "Groovy")]
    Groovy,
    #[strum(to_string = "Myghty")]
    Myghty,
    #[strum(to_string = "PureScript")]
    PureScript,
    #[strum(to_string = "TSQL")]
    Tsql,
    #[strum(to_string = "Java Properties")]
    JavaProperties,
    #[strum(to_string = "Ecere Projects")]
    EcereProjects,
    #[strum(to_string = "Type Language")]
    TypeLanguage,
    #[strum(to_string = "RAML")]
    Raml,
    #[strum(to_string = "Common Lisp")]
    CommonLisp,
    #[strum(to_string = "Creole")]
    Creole,
    #[strum(to_string = "Modelica")]
    Modelica,
    #[strum(to_string = "TeX")]
    TeX,
    #[strum(to_string = "Elixir")]
    Elixir,
    #[strum(to_string = "NetLink")]
    NetLink,
    #[strum(to_string = "Zig")]
    Zig,
    #[strum(to_string = "Riot")]
    Riot,
    #[strum(to_string = "PHP")]
    Php,
    #[strum(to_string = "OpenEdge ABL")]
    OpenEdgeAbl,
    #[strum(to_string = "AsciiDoc")]
    AsciiDoc,
    #[strum(to_string = "LookML")]
    LookMl,
    #[strum(to_string = "Source Pawn")]
    SourcePawn,
    #[strum(to_string = "Yacc")]
    Yacc,
    #[strum(to_string = "ApacheConf")]
    ApacheConf,
    #[strum(to_string = "TXL")]
    Txl,
    #[strum(to_string = "PowerShell")]
    PowerShell,
    #[strum(to_string = "GAML")]
    Gaml,
    #[strum(to_string = "XCompose")]
    XCompose,
    #[strum(to_string = "C#")]
    CSharp,
    #[strum(to_string = "Scilab")]
    Scilab,
    #[strum(to_string = "RobotFramework")]
    RobotFramework,
    #[strum(to_string = "Cython")]
    Cython,
    #[strum(to_string = "Emacs Lisp")]
    EmacsLisp,
    #[strum(to_string = "Metal")]
    Metal,
    #[strum(to_string = "JavaScript+ERB")]
    JavaScriptPlusErb,
    #[strum(to_string = "DataWeave")]
    DataWeave,
    #[strum(to_string = "Logtalk")]
    Logtalk,
    #[strum(to_string = "PLpgSQL")]
    PlPgSql,
    #[strum(to_string = "Augeas")]
    Augeas,
    #[strum(to_string = "MLIR")]
    Mlir,
    #[strum(to_string = "POV-Ray SDL")]
    PovRaySdl,
    #[strum(to_string = "Solidity")]
    Solidity,
    #[strum(to_string = "Mercury")]
    Mercury,
    #[strum(to_string = "Clojure")]
    Clojure,
    #[strum(to_string = "Genshi")]
    Genshi,
    #[strum(to_string = "Csound")]
    Csound,
    #[strum(to_string = "Monkey")]
    Monkey,
    #[strum(to_string = "Boo")]
    Boo,
    #[strum(to_string = "Unix Assembly")]
    UnixAssembly,
    #[strum(to_string = "MAXScript")]
    MaxScript,
    #[strum(to_string = "Rich Text Format")]
    RichTextFormat,
    #[strum(to_string = "Faust")]
    Faust,
    #[strum(to_string = "Erlang")]
    Erlang,
    #[strum(to_string = "Filterscript")]
    Filterscript,
    #[strum(to_string = "Stylus")]
    Stylus,
    #[strum(to_string = "mupad")]
    Mupad,
    #[strum(to_string = "Org")]
    Org,
    #[strum(to_string = "Glyph")]
    Glyph,
    #[strum(to_string = "SubRip Text")]
    SubRipText,
    #[strum(to_string = "Pod")]
    Pod,
    #[strum(to_string = "XQuery")]
    XQuery,
    #[strum(to_string = "CMake")]
    CMake,
    #[strum(to_string = "Gnuplot")]
    Gnuplot,
    #[strum(to_string = "Python traceback")]
    PythonTraceback,
    #[strum(to_string = "Darcs Patch")]
    DarcsPatch,
    #[strum(to_string = "ZAP")]
    Zap,
    #[strum(to_string = "Public Key")]
    PublicKey,
    #[strum(to_string = "Hy")]
    Hy,
    #[strum(to_string = "LFE")]
    Lfe,
    #[strum(to_string = "Wavefront Material")]
    WavefrontMaterial,
    #[strum(to_string = "AutoHotKey")]
    AutoHotKey,
    #[strum(to_string = "SVG")]
    Svg,
    #[strum(to_string = "Zimpl")]
    Zimpl,
    #[strum(to_string = "HiveQL")]
    HiveQL,
    #[strum(to_string = "J")]
    J,
    #[strum(to_string = "XPages")]
    XPages,
    #[strum(to_string = "Propeller Spin")]
    PropellerSpin,
    #[strum(to_string = "VBScript")]
    VbScript,
    #[strum(to_string = "reStructuredText")]
    ReStructuredText,
    #[strum(to_string = "Pic")]
    Pic,
    #[strum(to_string = "wdl")]
    Wdl,
    #[strum(to_string = "LTspice Symbol")]
    LTspiceSymbol,
    #[strum(to_string = "RPC")]
    Rpc,
    #[strum(to_string = "Vim script")]
    VimScript,
    #[strum(to_string = "TypeScript")]
    TypeScript,
    #[strum(to_string = "Dockerfile")]
    Dockerfile,
    #[strum(to_string = "QMake")]
    QMake,
    #[strum(to_string = "Git Config")]
    GitConfig,
    #[strum(to_string = "Reason")]
    Reason,
    #[strum(to_string = "Spline Font Database")]
    SplineFontDatabase,
    #[strum(to_string = "OCaml")]
    OCaml,
    #[strum(to_string = "MTML")]
    Mtml,
    #[strum(to_string = "Pike")]
    Pike,
    #[strum(to_string = "Objective-C")]
    ObjectiveC,
    #[strum(to_string = "ShellSession")]
    ShellSession,
    #[strum(to_string = "Meson")]
    Meson,
    #[strum(to_string = "Ox")]
    Ox,
    #[strum(to_string = "Module Management System")]
    ModuleManagementSystem,
    #[strum(to_string = "Coq")]
    Coq,
    #[strum(to_string = "X BitMap")]
    XBitMap,
    #[strum(to_string = "X Font Directory Index")]
    XFontDirectoryIndex,
    #[strum(to_string = "Terra")]
    Terra,
    #[strum(to_string = "SugarSS")]
    SugarSS,
    #[strum(to_string = "wisp")]
    Wisp,
    #[strum(to_string = "OpenQASM")]
    OpenQasm,
    #[strum(to_string = "Rascal")]
    Rascal,
    #[strum(to_string = "Chapel")]
    Chapel,
    #[strum(to_string = "C2hs Haskell")]
    C2hsHaskell,
    #[strum(to_string = "Cuda")]
    Cuda,
    #[strum(to_string = "LiveScript")]
    LiveScript,
    #[strum(to_string = "NCL")]
    Ncl,
    #[strum(to_string = "Haxe")]
    Haxe,
    #[strum(to_string = "AdobeFontMetrics")]
    AdobeFontMetrics,
    #[strum(to_string = "Java")]
    Java,
    #[strum(to_string = "SQLPL")]
    SqlPl,
    #[strum(to_string = "DM")]
    Dm,
    #[strum(to_string = "Opal")]
    Opal,
    #[strum(to_string = "CoffeeScript")]
    CoffeeScript,
    #[strum(to_string = "Open Policy Agent")]
    OpenPolicyAgent,
    #[strum(to_string = "Formatted")]
    Formatted,
    #[strum(to_string = "Roff")]
    Roff,
    #[strum(to_string = "Unified Parallel C")]
    UnifiedParallelC,
    #[strum(to_string = "Gerber Image")]
    GerberImage,
    #[strum(to_string = "BlitzMax")]
    BlitzMax,
    #[strum(to_string = "Moonscript")]
    Moonscript,
    #[strum(to_string = "Agda")]
    Agda,
    #[strum(to_string = "Tcl")]
    Tcl,
    #[strum(to_string = "Max")]
    Max,
    #[strum(to_string = "Hack")]
    Hack,
    #[strum(to_string = "Jison")]
    Jison,
    #[strum(to_string = "Click")]
    Click,
    #[strum(to_string = "Mako")]
    Mako,
    #[strum(to_string = "RUNOFF")]
    Runoff,
    #[strum(to_string = "MiniD")]
    MiniD,
    #[strum(to_string = "Odin")]
    Odin,
    #[strum(to_string = "RDoc")]
    RDoc,
    #[strum(to_string = "Cirru")]
    Cirru,
    #[strum(to_string = "HTML+ECR")]
    HtmlPlusEcr,
    #[strum(to_string = "CSS")]
    Css,
    #[strum(to_string = "Ada")]
    Ada,
    #[strum(to_string = "Omgrofl")]
    Omgrofl,
    #[strum(to_string = "Dart")]
    Dart,
    #[strum(to_string = "YAML")]
    Yaml,
    #[strum(to_string = "Clarion")]
    Clarion,
    #[strum(to_string = "KiCad Schematic")]
    KiCadSchematic,
    #[strum(to_string = "NPM Config")]
    NpmConfig,
    #[strum(to_string = "1C Enterprise")]
    OneCEnterprise,
    #[strum(to_string = "Linux Kernel Module")]
    LinuxKernelModule,
    #[strum(to_string = "Dylan")]
    Dylan,
    #[strum(to_string = "Gn")]
    Gn,
    #[strum(to_string = "Redcode")]
    Redcode,
    #[strum(to_string = "Eagle")]
    Eagle,
    #[strum(to_string = "VCL")]
    Vcl,
    #[strum(to_string = "LabVIEW")]
    LabView,
    #[strum(to_string = "Parrot Assembly")]
    ParrotAssembly,
    #[strum(to_string = "Graphviz (DOT)")]
    GraphvizDot,
    #[strum(to_string = "xBase")]
    XBase,
    #[strum(to_string = "ComponentPascal")]
    ComponentPascal,
    #[strum(to_string = "Ninja")]
    Ninja,
    #[strum(to_string = "Prisma")]
    Prisma,
    #[strum(to_string = "XS")]
    Xs,
    #[strum(to_string = "Clean")]
    Clean,
    #[strum(to_string = "Charity")]
    Charity,
    #[strum(to_string = "Protocol Buffer")]
    ProtocolBuffer,
    #[strum(to_string = "Kit")]
    Kit,
    #[strum(to_string = "D")]
    D,
    #[strum(to_string = "Bison")]
    Bison,
    #[strum(to_string = "Filebench WML")]
    FilebenchWml,
    #[strum(to_string = "Limbo")]
    Limbo,
    #[strum(to_string = "Glyph Bitmap Distribution Format")]
    GlyphBitmapDistributionFormat,
    #[strum(to_string = "Wget Config")]
    WgetConfig,
    #[strum(to_string = "Haskell")]
    Haskell,
    #[strum(to_string = "GDScript")]
    GdScript,
    #[strum(to_string = "GDB")]
    Gdb,
    #[strum(to_string = "PicoLisp")]
    PicoLisp,
    #[strum(to_string = "FreeMarker")]
    FreeMarker,
    #[strum(to_string = "Apollo Guidance Computer")]
    ApolloGuidanceComputer,
    #[strum(to_string = "Gentoo Eclass")]
    GentooEclass,
    #[strum(to_string = "PowerBuilder")]
    PowerBuilder,
    #[strum(to_string = "AspectJ")]
    AspectJ,
    #[strum(to_string = "Literate CoffeeScript")]
    LiterateCoffeeScript,
    #[strum(to_string = "Squirrel")]
    Squirrel,
    #[strum(to_string = "ooc")]
    Ooc,
    #[strum(to_string = "Pascal")]
    Pascal,
    #[strum(to_string = "NSIS")]
    Nsis,
    #[strum(to_string = "Csound Document")]
    CsoundDocument,
    #[strum(to_string = "Sass")]
    Sass,
    #[strum(to_string = "Graph Modeling Language")]
    GraphModelingLanguage,
    #[strum(to_string = "Twig")]
    Twig,
    #[strum(to_string = "HolyC")]
    HolyC,
    #[strum(to_string = "OpenType Feature File")]
    OpenTypeFeatureFile,
    #[strum(to_string = "XML Property List")]
    XmlPropertyList,
    #[strum(to_string = "Pawn")]
    Pawn,
    #[strum(to_string = "C")]
    C,
    #[strum(to_string = "SmPL")]
    SmPl,
    #[strum(to_string = "NL")]
    Nl,
    #[strum(to_string = "NetLogo")]
    NetLogo,
    #[strum(to_string = "Cpp-ObjDump")]
    CppObjDump,
    #[strum(to_string = "JSON5")]
    Json5,
    #[strum(to_string = "Proguard")]
    Proguard,
    #[strum(to_string = "ABNF")]
    Abnf,
    #[strum(to_string = "PureBasic")]
    PureBasic,
    #[strum(to_string = "XProc")]
    XProc,
    #[strum(to_string = "GraphQL")]
    GraphQl,
    #[strum(to_string = "Vala")]
    Vala,
    #[strum(to_string = "NASL")]
    Nasl,
    #[strum(to_string = "Perl")]
    Perl,
    #[strum(to_string = "Haml")]
    Haml,
    #[strum(to_string = "Literate Agda")]
    LiterateAgda,
    #[strum(to_string = "Liquid")]
    Liquid,
    #[strum(to_string = "RenderScript")]
    RenderScript,
    #[strum(to_string = "Literate Haskell")]
    LiterateHaskell,
    #[strum(to_string = "PogoScript")]
    PogoScript,
    #[strum(to_string = "World of Warcraft Addon Data")]
    WorldOfWarcraftAddonData,
    #[strum(to_string = "fish")]
    Fish,
    #[strum(to_string = "Nit")]
    Nit,
    #[strum(to_string = "WebVTT")]
    WebVtt,
    #[strum(to_string = "RMarkdown")]
    RMarkdown,
    #[strum(to_string = "GCC Machine Description")]
    GccMachineDescription,
    #[strum(to_string = "EJS")]
    Ejs,
    #[strum(to_string = "Lasso")]
    Lasso,
    #[strum(to_string = "Processing")]
    Processing,
    #[strum(to_string = "Closure Templates")]
    ClosureTemplates,
    #[strum(to_string = "PigLatin")]
    PigLatin,
    #[strum(to_string = "Xtend")]
    Xtend,
    #[strum(to_string = "TOML")]
    Toml,
    #[strum(to_string = "NetLinx+ERB")]
    NetLinkxPlusERB,
    #[strum(to_string = "Easybuild")]
    Easybuild,
    #[strum(to_string = "4D")]
    FourD,
    #[strum(to_string = "Cabal Config")]
    CabalConfig,
    #[strum(to_string = "Microsoft Developer Studio Project")]
    MicrosoftDeveloperStudioProject,
    #[strum(to_string = "Nextflow")]
    Nextflow,
    #[strum(to_string = "X PixMap")]
    XPixMap,
    #[strum(to_string = "XSLT")]
    Xslt,
    #[strum(to_string = "Textile")]
    Textfile,
    #[strum(to_string = "M")]
    M,
    #[strum(to_string = "JSONiq")]
    JsonIq,
    #[strum(to_string = "KiCad Legacy Layout")]
    KiCadLegacyLayout,
    #[strum(to_string = "Dogescript")]
    Dogescript,
    #[strum(to_string = "Jsonnet")]
    Jsonnet,
    #[strum(to_string = "Ragel")]
    Ragel,
    #[strum(to_string = "Uno")]
    Uno,
    #[strum(to_string = "HXML")]
    Hxml,
    #[strum(to_string = "HLSL")]
    Hlsl,
    #[strum(to_string = "ATS")]
    Ats,
    #[strum(to_string = "Eiffel")]
    Eiffel,
    #[strum(to_string = "Linker Script")]
    LinkerScript,
    #[strum(to_string = "Tea")]
    Tea,
    #[strum(to_string = "Quake")]
    Quake,
    #[strum(to_string = "Kotlin")]
    Kotlin,
    #[strum(to_string = "Puppet")]
    Puppet,
    #[strum(to_string = "Vue")]
    Vue,
    #[strum(to_string = "Parrot")]
    Parrot,
    #[strum(to_string = "Ioke")]
    Ioke,
    #[strum(to_string = "Lua")]
    Lua,
    #[strum(to_string = "SQF")]
    Sqf,
    #[strum(to_string = "MQL4")]
    Mql4,
    #[strum(to_string = "XML")]
    Xml,
    #[strum(to_string = "Red")]
    Red,
    #[strum(to_string = "Moocode")]
    Moocode,
    #[strum(to_string = "Julia")]
    Julia,
    #[strum(to_string = "Raw token data")]
    RawTokenData,
    #[strum(to_string = "Smalltalk")]
    Smalltalk,
    #[strum(to_string = "M4Sugar")]
    M4Sugar,
    #[strum(to_string = "ZIL")]
    Zil,
    #[strum(to_string = "mcfunction")]
    Mcfunction,
    #[strum(to_string = "ColdFusion CFC")]
    ColdFusionCfc,
    #[strum(to_string = "AppleScript")]
    AppleScript,
    #[strum(to_string = "E")]
    E,
    #[strum(to_string = "EQ")]
    Eq,
    #[strum(to_string = "Groovy Server Pages")]
    GroovyServerPages,
    #[strum(to_string = "ObjDump")]
    ObjDump,
    #[strum(to_string = "Ruby")]
    Ruby,
    #[strum(to_string = "Visual Basic .NET")]
    VisualBasicDotNet,
    #[strum(to_string = "Thrift")]
    Thrift,
    #[strum(to_string = "IGOR Pro")]
    IgorPro,
    #[strum(to_string = "Asymptote")]
    Asymptote,
    #[strum(to_string = "GLSL")]
    Glsl,
    #[strum(to_string = "Nearly")]
    Nearly,
    #[strum(to_string = "SSH Config")]
    SshConfig,
    #[strum(to_string = "Shell")]
    Shell,
    #[strum(to_string = "CSV")]
    Csv,
    #[strum(to_string = "edn")]
    Edn,
    #[strum(to_string = "HTML+ERB")]
    HtmlPlusErb,
    #[strum(to_string = "Ceylon")]
    Ceylon,
    #[strum(to_string = "Lex")]
    Lex,
    #[strum(to_string = "CartoCSS")]
    CartoCss,
    #[strum(to_string = "EmberScript")]
    EmberScript,
    #[strum(to_string = "JSONLD")]
    JsonLd,
    #[strum(to_string = "Pickle")]
    Pickle,
    #[strum(to_string = "Prolog")]
    Prolog,
    #[strum(to_string = "TI Program")]
    TiProgram,
    #[strum(to_string = "AutoIt")]
    AutoIt,
    #[strum(to_string = "ObjectScript")]
    ObjectScript,
    #[strum(to_string = "Csound Score")]
    CsoundScore,
    #[strum(to_string = "Papyrus")]
    Papyrus,
    #[strum(to_string = "Turtle")]
    Turtle,
    #[strum(to_string = "YARA")]
    Yara,
    #[strum(to_string = "Cap'n Proto")]
    CapnProto,
    #[strum(to_string = "PostCSS")]
    PostCss,
    #[strum(to_string = "UrWeb")]
    UrWeb,
    #[strum(to_string = "Muse")]
    Muse,
    #[strum(to_string = "MUF")]
    Muf,
    #[strum(to_string = "Alpine Abuild")]
    AlpineAbuild,
    #[strum(to_string = "C-ObjDump")]
    CObjDump,
    #[strum(to_string = "HTML")]
    Html,
    #[strum(to_string = "Rust")]
    Rust,
    #[strum(to_string = "Frege")]
    Frege,
    #[strum(to_string = "Isabelle ROOT")]
    IsabelleRoot,
    #[strum(to_string = "Windows Registry Entries")]
    WindowsRegistryEntries,
    #[strum(to_string = "Tcsh")]
    Tcsh,
    #[strum(to_string = "Racket")]
    Racket,
    #[strum(to_string = "Slim")]
    Slim,
    #[strum(to_string = "HTML+PHP")]
    HtmlPlusPhp,
    #[strum(to_string = "Fantom")]
    Fantom,
    #[strum(to_string = "Jupyter Notebook")]
    JupyterNotebook,
    #[strum(to_string = "HTTP")]
    Http,
    #[strum(to_string = "OpenStep Property List")]
    OpenStepPropertyList,
    #[strum(to_string = "BlitzBasic")]
    BlitzBasic,
    #[strum(to_string = "Batchfile")]
    Batchfile,
}
