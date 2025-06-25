use {
    crate::{
        artifact_id::ArtifactId,
        error::{EmbeddingError, InputManifestError},
        hash_algorithm::HashAlgorithm,
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    bindet::FileType as BinaryType,
    std::{
        collections::HashMap,
        fmt::{Debug, Display},
        fs::OpenOptions,
        io::Write as _,
        ops::Not,
        path::Path,
        str::FromStr,
        sync::LazyLock,
    },
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum KnownProgLang {
    FStar,
    GitAttributes,
    IrcLog,
    Eclipse,
    Blade,
    PureData,
    GrammaticalFramework,
    Brightscript,
    Go,
    BitBake,
    Cobol,
    Dhall,
    OpenCl,
    Pod6,
    Scheme,
    DigitalCommandLanguage,
    Diff,
    Motorola68KAssembly,
    Io,
    SRecodeTemplate,
    Scss,
    Ampl,
    RegularExpression,
    CoNllU,
    NanoRc,
    Slice,
    OpenScad,
    Jsx,
    Yang,
    FSharp,
    X10,
    CommonWorkflowLanguage,
    Stan,
    Befunge,
    GentooEbuild,
    AltiumDesigner,
    Alloy,
    Sage,
    ObjectDataInstanceNotation,
    ZenScript,
    AntBuildSystem,
    Pep8,
    BibTeX,
    Json,
    MavenPom,
    Smarty,
    WavefrontObject,
    Cpp,
    IgnoreList,
    DirectX3DFile,
    Apex,
    Llvm,
    UnrealScript,
    Tla,
    Hcl,
    EdjeDataCollection,
    Fancy,
    Krl,
    Ecl,
    RoffManpage,
    ObjectiveCpp,
    Turing,
    ShaderLab,
    Makefile,
    Factor,
    CWeb,
    Lsl,
    Pony,
    Gherkin,
    Awk,
    Cycript,
    Jolie,
    Ballerina,
    Starlark,
    GettextCatalog,
    Lolcode,
    Oz,
    PlantUml,
    Smt,
    V,
    Nix,
    Zeek,
    JisonLex,
    CurlConfig,
    KiCadLayout,
    HtmlPlusEex,
    Mask,
    Isabelle,
    LoomScript,
    Raku,
    Vba,
    Asn1,
    Brainfuck,
    DTrace,
    Elm,
    Crystal,
    Fortran,
    ChucK,
    HyPhy,
    Genie,
    Mql5,
    PlSql,
    Jasmin,
    Desktop,
    SelfLang,
    Swift,
    MIrcScript,
    Smali,
    Ec,
    JavaServerPages,
    Lean,
    Oxygene,
    Ston,
    InnoSetup,
    Asp,
    Gosu,
    Gradle,
    ObjectiveJ,
    Volt,
    Xc,
    Golo,
    EditorConfig,
    GCode,
    Mirah,
    PythonConsole,
    Inform7,
    Vhdl,
    M4,
    Q,
    ColdFusion,
    Antlr,
    RenPy,
    Eml,
    Tsx,
    NumPy,
    NesC,
    Modula3,
    NewLisp,
    Pug,
    Grace,
    Idris,
    YaSnippet,
    ParrotInternalRepresentation,
    SaltStackm,
    Rexx,
    CloudFirestoreSecurityRules,
    Ini,
    Matlab,
    Svelte,
    Text,
    Arc,
    Xojo,
    HtmlPlusDjango,
    AngelScript,
    Slash,
    Zephir,
    VimSmippet,
    SuperCollider,
    Modula2,
    WebOntologyLanguage,
    Handlebars,
    Collada,
    HtmlPlusRazor,
    Sparql,
    Texinfo,
    Stata,
    Opa,
    Ring,
    WebAssembly,
    Abap,
    Flex,
    Nim,
    Gams,
    Sas,
    Sql,
    Cson,
    Harbour,
    RHtml,
    Nemerle,
    Idl,
    JsonWithComments,
    ActionScript,
    MediaWiki,
    Nginx,
    Less,
    Forth,
    OpenRcRunscript,
    Apl,
    Python,
    Scaml,
    Verilog,
    Qml,
    Assembly,
    Marko,
    DObjDump,
    P4,
    Sed,
    Rouge,
    Pan,
    DnsZone,
    RpmSpec,
    JavaScript,
    SystemVerilog,
    Wollok,
    R,
    Latte,
    Mathematica,
    Ebnf,
    Rebol,
    Swig,
    JFlex,
    FigletFont,
    Dircolors,
    Shen,
    Scala,
    AgsScript,
    LilyPond,
    StandardMl,
    Bluespec,
    Unity3DAsset,
    Markdown,
    ApiBlueprint,
    Logos,
    ReadlineConfig,
    Cool,
    RealBasic,
    CodeQl,
    WebIdl,
    Clips,
    Gap,
    Nu,
    GameMakerLanguage,
    PostScript,
    HaProxy,
    Groovy,
    Myghty,
    PureScript,
    Tsql,
    JavaProperties,
    EcereProjects,
    TypeLanguage,
    Raml,
    CommonLisp,
    Creole,
    Modelica,
    TeX,
    Elixir,
    NetLink,
    Zig,
    Riot,
    Php,
    OpenEdgeAbl,
    AsciiDoc,
    LookMl,
    SourcePawn,
    Yacc,
    ApacheConf,
    Txl,
    PowerShell,
    Gaml,
    XCompose,
    CSharp,
    Scilab,
    RobotFramework,
    Cython,
    EmacsLisp,
    Metal,
    JavaScriptPlusErb,
    DataWeave,
    Logtalk,
    PlPgSql,
    Augeas,
    Mlir,
    PovRaySdl,
    Solidity,
    Mercury,
    Clojure,
    Genshi,
    Csound,
    Monkey,
    Boo,
    UnixAssembly,
    MaxScript,
    RichTextFormat,
    Faust,
    Erlang,
    Filterscript,
    Stylus,
    Mupad,
    Org,
    Glyph,
    SubRipText,
    Pod,
    XQuery,
    CMake,
    Gnuplot,
    PythonTraceback,
    DarcsPatch,
    Zap,
    PublicKey,
    Hy,
    Lfe,
    WavefrontMaterial,
    AutoHotKey,
    Svg,
    Zimpl,
    HiveQL,
    J,
    XPages,
    PropellerSpin,
    VbScript,
    ReStructuredText,
    Pic,
    Wdl,
    LTspiceSymbol,
    Rpc,
    VimScript,
    TypeScript,
    Dockerfile,
    QMake,
    GitConfig,
    Reason,
    SplineFontDatabase,
    OCaml,
    Mtml,
    Pike,
    ObjectiveC,
    ShellSession,
    Meson,
    Ox,
    ModuleManagementSystem,
    Coq,
    XBitMap,
    XFontDirectoryIndex,
    Terra,
    SugarSS,
    Wisp,
    OpenQasm,
    Rascal,
    Chapel,
    C2hsHaskell,
    Cuda,
    LiveScript,
    Ncl,
    Haxe,
    AdobeFontMetrics,
    Java,
    SqlPl,
    Dm,
    Opal,
    CoffeeScript,
    OpenPolicyAgent,
    Formatted,
    Roff,
    UnifiedParallelC,
    GerberImage,
    BlitzMax,
    Moonscript,
    Agda,
    Tcl,
    Max,
    Hack,
    Jison,
    Click,
    Mako,
    Runoff,
    MiniD,
    Odin,
    RDoc,
    Cirru,
    HtmlPlusEcr,
    Css,
    Ada,
    Omgrofl,
    Dart,
    Yaml,
    Clarion,
    KiCadSchematic,
    NpmConfig,
    OneCEnterprise,
    LinuxKernelModule,
    Dylan,
    Gn,
    Redcode,
    Eagle,
    Vcl,
    LabView,
    ParrotAssembly,
    GraphvizDot,
    XBase,
    ComponentPascal,
    Ninja,
    Prisma,
    Xs,
    Clean,
    Charity,
    ProtocolBuffer,
    Kit,
    D,
    Bison,
    FilebenchWml,
    Limbo,
    GlyphBitmapDistributionFormat,
    WgetConfig,
    Haskell,
    GdScript,
    Gdb,
    PicoLisp,
    FreeMarker,
    ApolloGuidanceComputer,
    GentooEclass,
    PowerBuilder,
    AspectJ,
    LiterateCoffeeScript,
    Squirrel,
    Ooc,
    Pascal,
    Nsis,
    CsoundDocument,
    Sass,
    GraphModelingLanguage,
    Twig,
    HolyC,
    OpenTypeFeatureFile,
    XmlPropertyList,
    Pawn,
    C,
    SmPl,
    Nl,
    NetLogo,
    CppObjDump,
    Json5,
    Proguard,
    Abnf,
    PureBasic,
    XProc,
    GraphQl,
    Vala,
    Nasl,
    Perl,
    Haml,
    LiterateAgda,
    Liquid,
    RenderScript,
    LiterateHaskell,
    PogoScript,
    WorldOfWarcraftAddonData,
    Fish,
    Nit,
    WebVtt,
    RMarkdown,
    GccMachineDescription,
    Ejs,
    Lasso,
    Processing,
    ClosureTemplates,
    PigLatin,
    Xtend,
    Toml,
    NetLinkxPlusERB,
    Easybuild,
    FourD,
    CabalConfig,
    MicrosoftDeveloperStudioProject,
    Nextflow,
    XPixMap,
    Xslt,
    Textfile,
    M,
    JsonIq,
    KiCadLegacyLayout,
    Dogescript,
    Jsonnet,
    Ragel,
    Uno,
    Hxml,
    Hlsl,
    Ats,
    Eiffel,
    LinkerScript,
    Tea,
    Quake,
    Kotlin,
    Puppet,
    Vue,
    Parrot,
    Ioke,
    Lua,
    Sqf,
    Mql4,
    Xml,
    Red,
    Moocode,
    Julia,
    RawTokenData,
    Smalltalk,
    M4Sugar,
    Zil,
    Mcfunction,
    ColdFusionCfc,
    AppleScript,
    E,
    Eq,
    GroovyServerPages,
    ObjDump,
    Ruby,
    VisualBasicDotNet,
    Thrift,
    IgorPro,
    Asymptote,
    Glsl,
    Nearly,
    SshConfig,
    Shell,
    Csv,
    Edn,
    HtmlPlusErb,
    Ceylon,
    Lex,
    CartoCss,
    EmberScript,
    JsonLd,
    Pickle,
    Prolog,
    TiProgram,
    AutoIt,
    ObjectScript,
    CsoundScore,
    Papyrus,
    Turtle,
    Yara,
    CapnProto,
    PostCss,
    UrWeb,
    Muse,
    Muf,
    AlpineAbuild,
    CObjDump,
    Html,
    Rust,
    Frege,
    IsabelleRoot,
    WindowsRegistryEntries,
    Tcsh,
    Racket,
    Slim,
    HtmlPlusPhp,
    Fantom,
    JupyterNotebook,
    Http,
    OpenStepPropertyList,
    BlitzBasic,
    Batchfile,
}

static LANG_NAMES: LazyLock<HashMap<KnownProgLang, &'static str>> = LazyLock::new(|| {
    let mut lang_names = HashMap::new();

    lang_names.insert(KnownProgLang::FStar, "F*");
    lang_names.insert(KnownProgLang::GitAttributes, "Git Attributes");
    lang_names.insert(KnownProgLang::IrcLog, "IRC Log");
    lang_names.insert(KnownProgLang::Eclipse, "ECLiPSe");
    lang_names.insert(KnownProgLang::Blade, "Blade");
    lang_names.insert(KnownProgLang::PureData, "Pure Data");
    lang_names.insert(KnownProgLang::GrammaticalFramework, "Grammatical Framework");
    lang_names.insert(KnownProgLang::Brightscript, "Brightscript");
    lang_names.insert(KnownProgLang::Go, "Go");
    lang_names.insert(KnownProgLang::BitBake, "BitBake");
    lang_names.insert(KnownProgLang::Cobol, "COBOL");
    lang_names.insert(KnownProgLang::Dhall, "Dhall");
    lang_names.insert(KnownProgLang::OpenCl, "OpenCL");
    lang_names.insert(KnownProgLang::Pod6, "Pod 6");
    lang_names.insert(KnownProgLang::Scheme, "Scheme");
    lang_names.insert(
        KnownProgLang::DigitalCommandLanguage,
        "DIGITAL Command Language",
    );
    lang_names.insert(KnownProgLang::Diff, "Diff");
    lang_names.insert(KnownProgLang::Motorola68KAssembly, "Motorola 68K Assembly");
    lang_names.insert(KnownProgLang::Io, "Io");
    lang_names.insert(KnownProgLang::SRecodeTemplate, "SRecode Template");
    lang_names.insert(KnownProgLang::Scss, "SCSS");
    lang_names.insert(KnownProgLang::Ampl, "AMPL");
    lang_names.insert(KnownProgLang::RegularExpression, "Regular Expression");
    lang_names.insert(KnownProgLang::CoNllU, "CoNLL-U");
    lang_names.insert(KnownProgLang::NanoRc, "nanorc");
    lang_names.insert(KnownProgLang::Slice, "Slice");
    lang_names.insert(KnownProgLang::OpenScad, "OpenSCAD");
    lang_names.insert(KnownProgLang::Jsx, "JSX");
    lang_names.insert(KnownProgLang::Yang, "YANG");
    lang_names.insert(KnownProgLang::FSharp, "F#");
    lang_names.insert(KnownProgLang::X10, "X10");
    lang_names.insert(
        KnownProgLang::CommonWorkflowLanguage,
        "Common Workflow Language",
    );
    lang_names.insert(KnownProgLang::Stan, "Stan");
    lang_names.insert(KnownProgLang::Befunge, "Befunge");
    lang_names.insert(KnownProgLang::GentooEbuild, "Gentoo Ebuild");
    lang_names.insert(KnownProgLang::AltiumDesigner, "Altium Designer");
    lang_names.insert(KnownProgLang::Alloy, "Alloy");
    lang_names.insert(KnownProgLang::Sage, "Sage");
    lang_names.insert(
        KnownProgLang::ObjectDataInstanceNotation,
        "Object Data Instance Notation",
    );
    lang_names.insert(KnownProgLang::ZenScript, "ZenScript");
    lang_names.insert(KnownProgLang::AntBuildSystem, "AntBuildSystem");
    lang_names.insert(KnownProgLang::Pep8, "Pep8");
    lang_names.insert(KnownProgLang::BibTeX, "BibTeX");
    lang_names.insert(KnownProgLang::Json, "JSON");
    lang_names.insert(KnownProgLang::MavenPom, "Maven POM");
    lang_names.insert(KnownProgLang::Smarty, "Smarty");
    lang_names.insert(KnownProgLang::WavefrontObject, "Wavefront Object");
    lang_names.insert(KnownProgLang::Cpp, "C++");
    lang_names.insert(KnownProgLang::IgnoreList, "Ignore List");
    lang_names.insert(KnownProgLang::DirectX3DFile, "DirectX 3D File");
    lang_names.insert(KnownProgLang::Apex, "Apex");
    lang_names.insert(KnownProgLang::Llvm, "LLVM");
    lang_names.insert(KnownProgLang::UnrealScript, "UnrealScript");
    lang_names.insert(KnownProgLang::Tla, "TLA");
    lang_names.insert(KnownProgLang::Hcl, "HCL");
    lang_names.insert(KnownProgLang::EdjeDataCollection, "Edje Data Collection");
    lang_names.insert(KnownProgLang::Fancy, "Fancy");
    lang_names.insert(KnownProgLang::Krl, "KRL");
    lang_names.insert(KnownProgLang::Ecl, "ECL");
    lang_names.insert(KnownProgLang::RoffManpage, "Roff Manpage");
    lang_names.insert(KnownProgLang::ObjectiveCpp, "Objective-C++");
    lang_names.insert(KnownProgLang::Turing, "Turing");
    lang_names.insert(KnownProgLang::ShaderLab, "ShaderLab");
    lang_names.insert(KnownProgLang::Makefile, "Makefile");
    lang_names.insert(KnownProgLang::Factor, "Factor");
    lang_names.insert(KnownProgLang::CWeb, "CWeb");
    lang_names.insert(KnownProgLang::Lsl, "LSL");
    lang_names.insert(KnownProgLang::Pony, "Pony");
    lang_names.insert(KnownProgLang::Gherkin, "Gherkin");
    lang_names.insert(KnownProgLang::Awk, "Awk");
    lang_names.insert(KnownProgLang::Cycript, "Cycript");
    lang_names.insert(KnownProgLang::Jolie, "Jolie");
    lang_names.insert(KnownProgLang::Ballerina, "Ballerina");
    lang_names.insert(KnownProgLang::Starlark, "Starlark");
    lang_names.insert(KnownProgLang::GettextCatalog, "Gettext Catalog");
    lang_names.insert(KnownProgLang::Lolcode, "Lolcode");
    lang_names.insert(KnownProgLang::Oz, "Oz");
    lang_names.insert(KnownProgLang::PlantUml, "PlantUML");
    lang_names.insert(KnownProgLang::Smt, "SMT");
    lang_names.insert(KnownProgLang::V, "V");
    lang_names.insert(KnownProgLang::Nix, "Nix");
    lang_names.insert(KnownProgLang::Zeek, "Zeek");
    lang_names.insert(KnownProgLang::JisonLex, "Jison Lex");
    lang_names.insert(KnownProgLang::CurlConfig, "cURL Config");
    lang_names.insert(KnownProgLang::KiCadLayout, "KiCad Layout");
    lang_names.insert(KnownProgLang::HtmlPlusEex, "HTML+EEX");
    lang_names.insert(KnownProgLang::Mask, "Mask");
    lang_names.insert(KnownProgLang::Isabelle, "Isabelle");
    lang_names.insert(KnownProgLang::LoomScript, "LoomScript");
    lang_names.insert(KnownProgLang::Raku, "Raku");
    lang_names.insert(KnownProgLang::Vba, "VBA");
    lang_names.insert(KnownProgLang::Asn1, "ASN.1");
    lang_names.insert(KnownProgLang::Brainfuck, "Brainfuck");
    lang_names.insert(KnownProgLang::DTrace, "DTrace");
    lang_names.insert(KnownProgLang::Elm, "Elm");
    lang_names.insert(KnownProgLang::Crystal, "Crystal");
    lang_names.insert(KnownProgLang::Fortran, "Fortran");
    lang_names.insert(KnownProgLang::ChucK, "ChucK");
    lang_names.insert(KnownProgLang::HyPhy, "HyPhy");
    lang_names.insert(KnownProgLang::Genie, "Genie");
    lang_names.insert(KnownProgLang::Mql5, "MQL5");
    lang_names.insert(KnownProgLang::PlSql, "PLSQL");
    lang_names.insert(KnownProgLang::Jasmin, "Jasmin");
    lang_names.insert(KnownProgLang::Desktop, "desktop");
    lang_names.insert(KnownProgLang::SelfLang, "Self");
    lang_names.insert(KnownProgLang::Swift, "Swift");
    lang_names.insert(KnownProgLang::MIrcScript, "mIRC Script");
    lang_names.insert(KnownProgLang::Smali, "Smali");
    lang_names.insert(KnownProgLang::Ec, "eC");
    lang_names.insert(KnownProgLang::JavaServerPages, "JavaServerPages");
    lang_names.insert(KnownProgLang::Lean, "Lean");
    lang_names.insert(KnownProgLang::Oxygene, "Oxygene");
    lang_names.insert(KnownProgLang::Ston, "STON");
    lang_names.insert(KnownProgLang::InnoSetup, "Inno Setup");
    lang_names.insert(KnownProgLang::Asp, "ASP");
    lang_names.insert(KnownProgLang::Gosu, "Gosu");
    lang_names.insert(KnownProgLang::Gradle, "Gradle");
    lang_names.insert(KnownProgLang::ObjectiveJ, "Objective-J");
    lang_names.insert(KnownProgLang::Volt, "Volt");
    lang_names.insert(KnownProgLang::Xc, "XC");
    lang_names.insert(KnownProgLang::Golo, "Golo");
    lang_names.insert(KnownProgLang::EditorConfig, "EditorConfig");
    lang_names.insert(KnownProgLang::GCode, "G-code");
    lang_names.insert(KnownProgLang::Mirah, "Mirah");
    lang_names.insert(KnownProgLang::PythonConsole, "Python console");
    lang_names.insert(KnownProgLang::Inform7, "Inform 7");
    lang_names.insert(KnownProgLang::Vhdl, "VHDL");
    lang_names.insert(KnownProgLang::M4, "M4");
    lang_names.insert(KnownProgLang::Q, "q");
    lang_names.insert(KnownProgLang::ColdFusion, "ColdFusion");
    lang_names.insert(KnownProgLang::Antlr, "ANTLR");
    lang_names.insert(KnownProgLang::RenPy, "Ren'Py");
    lang_names.insert(KnownProgLang::Eml, "EML");
    lang_names.insert(KnownProgLang::Tsx, "TSX");
    lang_names.insert(KnownProgLang::NumPy, "NumPy");
    lang_names.insert(KnownProgLang::NesC, "nesC");
    lang_names.insert(KnownProgLang::Modula3, "Modula-3");
    lang_names.insert(KnownProgLang::NewLisp, "NewLisp");
    lang_names.insert(KnownProgLang::Pug, "Pug");
    lang_names.insert(KnownProgLang::Grace, "Grace");
    lang_names.insert(KnownProgLang::Idris, "Idris");
    lang_names.insert(KnownProgLang::YaSnippet, "YASnippet");
    lang_names.insert(
        KnownProgLang::ParrotInternalRepresentation,
        "Parrot Internal Representation",
    );
    lang_names.insert(KnownProgLang::SaltStackm, "SaltStack");
    lang_names.insert(KnownProgLang::Rexx, "REXX");
    lang_names.insert(
        KnownProgLang::CloudFirestoreSecurityRules,
        "Cloud Firestore Security Rules",
    );
    lang_names.insert(KnownProgLang::Ini, "INI");
    lang_names.insert(KnownProgLang::Matlab, "MATLAB");
    lang_names.insert(KnownProgLang::Svelte, "Svelte");
    lang_names.insert(KnownProgLang::Text, "Text");
    lang_names.insert(KnownProgLang::Arc, "Arc");
    lang_names.insert(KnownProgLang::Xojo, "Xojo");
    lang_names.insert(KnownProgLang::HtmlPlusDjango, "HTML+Django");
    lang_names.insert(KnownProgLang::AngelScript, "AngelScript");
    lang_names.insert(KnownProgLang::Slash, "Slash");
    lang_names.insert(KnownProgLang::Zephir, "Zephir");
    lang_names.insert(KnownProgLang::VimSmippet, "Vim Snippet");
    lang_names.insert(KnownProgLang::SuperCollider, "SuperCollider");
    lang_names.insert(KnownProgLang::Modula2, "Modula-2");
    lang_names.insert(KnownProgLang::WebOntologyLanguage, "Web Ontology Language");
    lang_names.insert(KnownProgLang::Handlebars, "Handlebars");
    lang_names.insert(KnownProgLang::Collada, "COLLADA");
    lang_names.insert(KnownProgLang::HtmlPlusRazor, "HTML+Razor");
    lang_names.insert(KnownProgLang::Sparql, "SPARQL");
    lang_names.insert(KnownProgLang::Texinfo, "Texinfo");
    lang_names.insert(KnownProgLang::Stata, "Stata");
    lang_names.insert(KnownProgLang::Opa, "Opa");
    lang_names.insert(KnownProgLang::Ring, "Ring");
    lang_names.insert(KnownProgLang::WebAssembly, "WebAssembly");
    lang_names.insert(KnownProgLang::Abap, "ABAP");
    lang_names.insert(KnownProgLang::Flex, "FLUX");
    lang_names.insert(KnownProgLang::Nim, "Nim");
    lang_names.insert(KnownProgLang::Gams, "GAMS");
    lang_names.insert(KnownProgLang::Sas, "SAS");
    lang_names.insert(KnownProgLang::Sql, "SQL");
    lang_names.insert(KnownProgLang::Cson, "CSON");
    lang_names.insert(KnownProgLang::Harbour, "Harbour");
    lang_names.insert(KnownProgLang::RHtml, "RHTML");
    lang_names.insert(KnownProgLang::Nemerle, "Nemerle");
    lang_names.insert(KnownProgLang::Idl, "IDL");
    lang_names.insert(KnownProgLang::JsonWithComments, "JSON with Comments");
    lang_names.insert(KnownProgLang::ActionScript, "ActionScript");
    lang_names.insert(KnownProgLang::MediaWiki, "MediaWiki");
    lang_names.insert(KnownProgLang::Nginx, "Nginx");
    lang_names.insert(KnownProgLang::Less, "Less");
    lang_names.insert(KnownProgLang::Forth, "Forth");
    lang_names.insert(KnownProgLang::OpenRcRunscript, "OpenRC runscript");
    lang_names.insert(KnownProgLang::Apl, "APL");
    lang_names.insert(KnownProgLang::Python, "Python");
    lang_names.insert(KnownProgLang::Scaml, "Scaml");
    lang_names.insert(KnownProgLang::Verilog, "Verilog");
    lang_names.insert(KnownProgLang::Qml, "QML");
    lang_names.insert(KnownProgLang::Assembly, "Assembly");
    lang_names.insert(KnownProgLang::Marko, "Marko");
    lang_names.insert(KnownProgLang::DObjDump, "D-ObjDump");
    lang_names.insert(KnownProgLang::P4, "P4");
    lang_names.insert(KnownProgLang::Sed, "sed");
    lang_names.insert(KnownProgLang::Rouge, "Rouge");
    lang_names.insert(KnownProgLang::Pan, "Pan");
    lang_names.insert(KnownProgLang::DnsZone, "DNS Zone");
    lang_names.insert(KnownProgLang::RpmSpec, "RPM Spec");
    lang_names.insert(KnownProgLang::JavaScript, "JavaScript");
    lang_names.insert(KnownProgLang::SystemVerilog, "SystemVerilog");
    lang_names.insert(KnownProgLang::Wollok, "Wollok");
    lang_names.insert(KnownProgLang::R, "R");
    lang_names.insert(KnownProgLang::Latte, "Latte");
    lang_names.insert(KnownProgLang::Mathematica, "Mathematica");
    lang_names.insert(KnownProgLang::Ebnf, "EBNF");
    lang_names.insert(KnownProgLang::Rebol, "Rebol");
    lang_names.insert(KnownProgLang::Swig, "SWIG");
    lang_names.insert(KnownProgLang::JFlex, "JFlex");
    lang_names.insert(KnownProgLang::FigletFont, "FIGlet Font");
    lang_names.insert(KnownProgLang::Dircolors, "dircolors");
    lang_names.insert(KnownProgLang::Shen, "Shen");
    lang_names.insert(KnownProgLang::Scala, "Scala");
    lang_names.insert(KnownProgLang::AgsScript, "AGS Script");
    lang_names.insert(KnownProgLang::LilyPond, "LilyPond");
    lang_names.insert(KnownProgLang::StandardMl, "Standard ML");
    lang_names.insert(KnownProgLang::Bluespec, "Bluespec");
    lang_names.insert(KnownProgLang::Unity3DAsset, "Unity3D Asset");
    lang_names.insert(KnownProgLang::Markdown, "Markdown");
    lang_names.insert(KnownProgLang::ApiBlueprint, "API Blueprint");
    lang_names.insert(KnownProgLang::Logos, "Logos");
    lang_names.insert(KnownProgLang::ReadlineConfig, "Readline Config");
    lang_names.insert(KnownProgLang::Cool, "Cool");
    lang_names.insert(KnownProgLang::RealBasic, "REALbasic");
    lang_names.insert(KnownProgLang::CodeQl, "CodeQL");
    lang_names.insert(KnownProgLang::WebIdl, "WebIDL");
    lang_names.insert(KnownProgLang::Clips, "CLIPS");
    lang_names.insert(KnownProgLang::Gap, "GAP");
    lang_names.insert(KnownProgLang::Nu, "Nu");
    lang_names.insert(KnownProgLang::GameMakerLanguage, "Game Maker Language");
    lang_names.insert(KnownProgLang::PostScript, "PostScript");
    lang_names.insert(KnownProgLang::HaProxy, "HAProxy");
    lang_names.insert(KnownProgLang::Groovy, "Groovy");
    lang_names.insert(KnownProgLang::Myghty, "Myghty");
    lang_names.insert(KnownProgLang::PureScript, "PureScript");
    lang_names.insert(KnownProgLang::Tsql, "TSQL");
    lang_names.insert(KnownProgLang::JavaProperties, "Java Properties");
    lang_names.insert(KnownProgLang::EcereProjects, "Ecere Projects");
    lang_names.insert(KnownProgLang::TypeLanguage, "Type Language");
    lang_names.insert(KnownProgLang::Raml, "RAML");
    lang_names.insert(KnownProgLang::CommonLisp, "Common Lisp");
    lang_names.insert(KnownProgLang::Creole, "Creole");
    lang_names.insert(KnownProgLang::Modelica, "Modelica");
    lang_names.insert(KnownProgLang::TeX, "TeX");
    lang_names.insert(KnownProgLang::Elixir, "Elixir");
    lang_names.insert(KnownProgLang::NetLink, "NetLink");
    lang_names.insert(KnownProgLang::Zig, "Zig");
    lang_names.insert(KnownProgLang::Riot, "Riot");
    lang_names.insert(KnownProgLang::Php, "PHP");
    lang_names.insert(KnownProgLang::OpenEdgeAbl, "OpenEdge ABL");
    lang_names.insert(KnownProgLang::AsciiDoc, "AsciiDoc");
    lang_names.insert(KnownProgLang::LookMl, "LookML");
    lang_names.insert(KnownProgLang::SourcePawn, "Source Pawn");
    lang_names.insert(KnownProgLang::Yacc, "Yacc");
    lang_names.insert(KnownProgLang::ApacheConf, "ApacheConf");
    lang_names.insert(KnownProgLang::Txl, "TXL");
    lang_names.insert(KnownProgLang::PowerShell, "PowerShell");
    lang_names.insert(KnownProgLang::Gaml, "GAML");
    lang_names.insert(KnownProgLang::XCompose, "XCompose");
    lang_names.insert(KnownProgLang::CSharp, "C#");
    lang_names.insert(KnownProgLang::Scilab, "Scilab");
    lang_names.insert(KnownProgLang::RobotFramework, "RobotFramework");
    lang_names.insert(KnownProgLang::Cython, "Cython");
    lang_names.insert(KnownProgLang::EmacsLisp, "Emacs Lisp");
    lang_names.insert(KnownProgLang::Metal, "Metal");
    lang_names.insert(KnownProgLang::JavaScriptPlusErb, "JavaScript+ERB");
    lang_names.insert(KnownProgLang::DataWeave, "DataWeave");
    lang_names.insert(KnownProgLang::Logtalk, "Logtalk");
    lang_names.insert(KnownProgLang::PlPgSql, "PLpgSQL");
    lang_names.insert(KnownProgLang::Augeas, "Augeas");
    lang_names.insert(KnownProgLang::Mlir, "MLIR");
    lang_names.insert(KnownProgLang::PovRaySdl, "POV-Ray SDL");
    lang_names.insert(KnownProgLang::Solidity, "Solidity");
    lang_names.insert(KnownProgLang::Mercury, "Mercury");
    lang_names.insert(KnownProgLang::Clojure, "Clojure");
    lang_names.insert(KnownProgLang::Genshi, "Genshi");
    lang_names.insert(KnownProgLang::Csound, "Csound");
    lang_names.insert(KnownProgLang::Monkey, "Monkey");
    lang_names.insert(KnownProgLang::Boo, "Boo");
    lang_names.insert(KnownProgLang::UnixAssembly, "Unix Assembly");
    lang_names.insert(KnownProgLang::MaxScript, "MAXScript");
    lang_names.insert(KnownProgLang::RichTextFormat, "Rich Text Format");
    lang_names.insert(KnownProgLang::Faust, "Faust");
    lang_names.insert(KnownProgLang::Erlang, "Erlang");
    lang_names.insert(KnownProgLang::Filterscript, "Filterscript");
    lang_names.insert(KnownProgLang::Stylus, "Stylus");
    lang_names.insert(KnownProgLang::Mupad, "mupad");
    lang_names.insert(KnownProgLang::Org, "Org");
    lang_names.insert(KnownProgLang::Glyph, "Glyph");
    lang_names.insert(KnownProgLang::SubRipText, "SubRip Text");
    lang_names.insert(KnownProgLang::Pod, "Pod");
    lang_names.insert(KnownProgLang::XQuery, "XQuery");
    lang_names.insert(KnownProgLang::CMake, "CMake");
    lang_names.insert(KnownProgLang::Gnuplot, "Gnuplot");
    lang_names.insert(KnownProgLang::PythonTraceback, "Python traceback");
    lang_names.insert(KnownProgLang::DarcsPatch, "Darcs Patch");
    lang_names.insert(KnownProgLang::Zap, "ZAP");
    lang_names.insert(KnownProgLang::PublicKey, "Public Key");
    lang_names.insert(KnownProgLang::Hy, "Hy");
    lang_names.insert(KnownProgLang::Lfe, "LFE");
    lang_names.insert(KnownProgLang::WavefrontMaterial, "Wavefront Material");
    lang_names.insert(KnownProgLang::AutoHotKey, "AutoHotKey");
    lang_names.insert(KnownProgLang::Svg, "SVG");
    lang_names.insert(KnownProgLang::Zimpl, "Zimpl");
    lang_names.insert(KnownProgLang::HiveQL, "HiveQL");
    lang_names.insert(KnownProgLang::J, "J");
    lang_names.insert(KnownProgLang::XPages, "XPages");
    lang_names.insert(KnownProgLang::PropellerSpin, "Propeller Spin");
    lang_names.insert(KnownProgLang::VbScript, "VBScript");
    lang_names.insert(KnownProgLang::ReStructuredText, "reStructuredText");
    lang_names.insert(KnownProgLang::Pic, "Pic");
    lang_names.insert(KnownProgLang::Wdl, "wdl");
    lang_names.insert(KnownProgLang::LTspiceSymbol, "LTspice Symbol");
    lang_names.insert(KnownProgLang::Rpc, "RPC");
    lang_names.insert(KnownProgLang::VimScript, "Vim script");
    lang_names.insert(KnownProgLang::TypeScript, "TypeScript");
    lang_names.insert(KnownProgLang::Dockerfile, "Dockerfile");
    lang_names.insert(KnownProgLang::QMake, "QMake");
    lang_names.insert(KnownProgLang::GitConfig, "Git Config");
    lang_names.insert(KnownProgLang::Reason, "Reason");
    lang_names.insert(KnownProgLang::SplineFontDatabase, "Spline Font Database");
    lang_names.insert(KnownProgLang::OCaml, "OCaml");
    lang_names.insert(KnownProgLang::Mtml, "MTML");
    lang_names.insert(KnownProgLang::Pike, "Pike");
    lang_names.insert(KnownProgLang::ObjectiveC, "Objective-C");
    lang_names.insert(KnownProgLang::ShellSession, "ShellSession");
    lang_names.insert(KnownProgLang::Meson, "Meson");
    lang_names.insert(KnownProgLang::Ox, "Ox");
    lang_names.insert(
        KnownProgLang::ModuleManagementSystem,
        "Module Management System",
    );
    lang_names.insert(KnownProgLang::Coq, "Coq");
    lang_names.insert(KnownProgLang::XBitMap, "X BitMap");
    lang_names.insert(KnownProgLang::XFontDirectoryIndex, "X Font Directory Index");
    lang_names.insert(KnownProgLang::Terra, "Terra");
    lang_names.insert(KnownProgLang::SugarSS, "SugarSS");
    lang_names.insert(KnownProgLang::Wisp, "wisp");
    lang_names.insert(KnownProgLang::OpenQasm, "OpenQASM");
    lang_names.insert(KnownProgLang::Rascal, "Rascal");
    lang_names.insert(KnownProgLang::Chapel, "Chapel");
    lang_names.insert(KnownProgLang::C2hsHaskell, "C2hs Haskell");
    lang_names.insert(KnownProgLang::Cuda, "Cuda");
    lang_names.insert(KnownProgLang::LiveScript, "LiveScript");
    lang_names.insert(KnownProgLang::Ncl, "NCL");
    lang_names.insert(KnownProgLang::Haxe, "Haxe");
    lang_names.insert(KnownProgLang::AdobeFontMetrics, "AdobeFontMetrics");
    lang_names.insert(KnownProgLang::Java, "Java");
    lang_names.insert(KnownProgLang::SqlPl, "SQLPL");
    lang_names.insert(KnownProgLang::Dm, "DM");
    lang_names.insert(KnownProgLang::Opal, "Opal");
    lang_names.insert(KnownProgLang::CoffeeScript, "CoffeeScript");
    lang_names.insert(KnownProgLang::OpenPolicyAgent, "Open Policy Agent");
    lang_names.insert(KnownProgLang::Formatted, "Formatted");
    lang_names.insert(KnownProgLang::Roff, "Roff");
    lang_names.insert(KnownProgLang::UnifiedParallelC, "Unified Parallel C");
    lang_names.insert(KnownProgLang::GerberImage, "Gerber Image");
    lang_names.insert(KnownProgLang::BlitzMax, "BlitzMax");
    lang_names.insert(KnownProgLang::Moonscript, "Moonscript");
    lang_names.insert(KnownProgLang::Agda, "Agda");
    lang_names.insert(KnownProgLang::Tcl, "Tcl");
    lang_names.insert(KnownProgLang::Max, "Max");
    lang_names.insert(KnownProgLang::Hack, "Hack");
    lang_names.insert(KnownProgLang::Jison, "Jison");
    lang_names.insert(KnownProgLang::Click, "Click");
    lang_names.insert(KnownProgLang::Mako, "Mako");
    lang_names.insert(KnownProgLang::Runoff, "RUNOFF");
    lang_names.insert(KnownProgLang::MiniD, "MiniD");
    lang_names.insert(KnownProgLang::Odin, "Odin");
    lang_names.insert(KnownProgLang::RDoc, "RDoc");
    lang_names.insert(KnownProgLang::Cirru, "Cirru");
    lang_names.insert(KnownProgLang::HtmlPlusEcr, "HTML+ECR");
    lang_names.insert(KnownProgLang::Css, "CSS");
    lang_names.insert(KnownProgLang::Ada, "Ada");
    lang_names.insert(KnownProgLang::Omgrofl, "Omgrofl");
    lang_names.insert(KnownProgLang::Dart, "Dart");
    lang_names.insert(KnownProgLang::Yaml, "YAML");
    lang_names.insert(KnownProgLang::Clarion, "Clarion");
    lang_names.insert(KnownProgLang::KiCadSchematic, "KiCad Schematic");
    lang_names.insert(KnownProgLang::NpmConfig, "NPM Config");
    lang_names.insert(KnownProgLang::OneCEnterprise, "1C Enterprise");
    lang_names.insert(KnownProgLang::LinuxKernelModule, "Linux Kernel Module");
    lang_names.insert(KnownProgLang::Dylan, "Dylan");
    lang_names.insert(KnownProgLang::Gn, "Gn");
    lang_names.insert(KnownProgLang::Redcode, "Redcode");
    lang_names.insert(KnownProgLang::Eagle, "Eagle");
    lang_names.insert(KnownProgLang::Vcl, "VCL");
    lang_names.insert(KnownProgLang::LabView, "LabVIEW");
    lang_names.insert(KnownProgLang::ParrotAssembly, "Parrot Assembly");
    lang_names.insert(KnownProgLang::GraphvizDot, "Graphviz (DOT)");
    lang_names.insert(KnownProgLang::XBase, "xBase");
    lang_names.insert(KnownProgLang::ComponentPascal, "ComponentPascal");
    lang_names.insert(KnownProgLang::Ninja, "Ninja");
    lang_names.insert(KnownProgLang::Prisma, "Prisma");
    lang_names.insert(KnownProgLang::Xs, "XS");
    lang_names.insert(KnownProgLang::Clean, "Clean");
    lang_names.insert(KnownProgLang::Charity, "Charity");
    lang_names.insert(KnownProgLang::ProtocolBuffer, "Protocol Buffer");
    lang_names.insert(KnownProgLang::Kit, "Kit");
    lang_names.insert(KnownProgLang::D, "D");
    lang_names.insert(KnownProgLang::Bison, "Bison");
    lang_names.insert(KnownProgLang::FilebenchWml, "Filebench WML");
    lang_names.insert(KnownProgLang::Limbo, "Limbo");
    lang_names.insert(
        KnownProgLang::GlyphBitmapDistributionFormat,
        "Glyph Bitmap Distribution Format",
    );
    lang_names.insert(KnownProgLang::WgetConfig, "Wget Config");
    lang_names.insert(KnownProgLang::Haskell, "Haskell");
    lang_names.insert(KnownProgLang::GdScript, "GDScript");
    lang_names.insert(KnownProgLang::Gdb, "GDB");
    lang_names.insert(KnownProgLang::PicoLisp, "PicoLisp");
    lang_names.insert(KnownProgLang::FreeMarker, "FreeMarker");
    lang_names.insert(
        KnownProgLang::ApolloGuidanceComputer,
        "Apollo Guidance Computer",
    );
    lang_names.insert(KnownProgLang::GentooEclass, "Gentoo Eclass");
    lang_names.insert(KnownProgLang::PowerBuilder, "PowerBuilder");
    lang_names.insert(KnownProgLang::AspectJ, "AspectJ");
    lang_names.insert(KnownProgLang::LiterateCoffeeScript, "Literate CoffeeScript");
    lang_names.insert(KnownProgLang::Squirrel, "Squirrel");
    lang_names.insert(KnownProgLang::Ooc, "ooc");
    lang_names.insert(KnownProgLang::Pascal, "Pascal");
    lang_names.insert(KnownProgLang::Nsis, "NSIS");
    lang_names.insert(KnownProgLang::CsoundDocument, "Csound Document");
    lang_names.insert(KnownProgLang::Sass, "Sass");
    lang_names.insert(
        KnownProgLang::GraphModelingLanguage,
        "Graph Modeling Language",
    );
    lang_names.insert(KnownProgLang::Twig, "Twig");
    lang_names.insert(KnownProgLang::HolyC, "HolyC");
    lang_names.insert(KnownProgLang::OpenTypeFeatureFile, "OpenType Feature File");
    lang_names.insert(KnownProgLang::XmlPropertyList, "XML Property List");
    lang_names.insert(KnownProgLang::Pawn, "Pawn");
    lang_names.insert(KnownProgLang::C, "C");
    lang_names.insert(KnownProgLang::SmPl, "SmPL");
    lang_names.insert(KnownProgLang::Nl, "NL");
    lang_names.insert(KnownProgLang::NetLogo, "NetLogo");
    lang_names.insert(KnownProgLang::CppObjDump, "Cpp-ObjDump");
    lang_names.insert(KnownProgLang::Json5, "JSON5");
    lang_names.insert(KnownProgLang::Proguard, "Proguard");
    lang_names.insert(KnownProgLang::Abnf, "ABNF");
    lang_names.insert(KnownProgLang::PureBasic, "PureBasic");
    lang_names.insert(KnownProgLang::XProc, "XProc");
    lang_names.insert(KnownProgLang::GraphQl, "GraphQL");
    lang_names.insert(KnownProgLang::Vala, "Vala");
    lang_names.insert(KnownProgLang::Nasl, "NASL");
    lang_names.insert(KnownProgLang::Perl, "Perl");
    lang_names.insert(KnownProgLang::Haml, "Haml");
    lang_names.insert(KnownProgLang::LiterateAgda, "Literate Agda");
    lang_names.insert(KnownProgLang::Liquid, "Liquid");
    lang_names.insert(KnownProgLang::RenderScript, "RenderScript");
    lang_names.insert(KnownProgLang::LiterateHaskell, "Literate Haskell");
    lang_names.insert(KnownProgLang::PogoScript, "PogoScript");
    lang_names.insert(
        KnownProgLang::WorldOfWarcraftAddonData,
        "World of Warcraft Addon Data",
    );
    lang_names.insert(KnownProgLang::Fish, "fish");
    lang_names.insert(KnownProgLang::Nit, "Nit");
    lang_names.insert(KnownProgLang::WebVtt, "WebVTT");
    lang_names.insert(KnownProgLang::RMarkdown, "RMarkdown");
    lang_names.insert(
        KnownProgLang::GccMachineDescription,
        "GCC Machine Description",
    );
    lang_names.insert(KnownProgLang::Ejs, "EJS");
    lang_names.insert(KnownProgLang::Lasso, "Lasso");
    lang_names.insert(KnownProgLang::Processing, "Processing");
    lang_names.insert(KnownProgLang::ClosureTemplates, "Closure Templates");
    lang_names.insert(KnownProgLang::PigLatin, "PigLatin");
    lang_names.insert(KnownProgLang::Xtend, "Xtend");
    lang_names.insert(KnownProgLang::Toml, "TOML");
    lang_names.insert(KnownProgLang::NetLinkxPlusERB, "NetLinx+ERB");
    lang_names.insert(KnownProgLang::Easybuild, "Easybuild");
    lang_names.insert(KnownProgLang::FourD, "4D");
    lang_names.insert(KnownProgLang::CabalConfig, "Cabal Config");
    lang_names.insert(
        KnownProgLang::MicrosoftDeveloperStudioProject,
        "Microsoft Developer Studio Project",
    );
    lang_names.insert(KnownProgLang::Nextflow, "Nextflow");
    lang_names.insert(KnownProgLang::XPixMap, "X PixMap");
    lang_names.insert(KnownProgLang::Xslt, "XSLT");
    lang_names.insert(KnownProgLang::Textfile, "Textile");
    lang_names.insert(KnownProgLang::M, "M");
    lang_names.insert(KnownProgLang::JsonIq, "JSONiq");
    lang_names.insert(KnownProgLang::KiCadLegacyLayout, "KiCad Legacy Layout");
    lang_names.insert(KnownProgLang::Dogescript, "Dogescript");
    lang_names.insert(KnownProgLang::Jsonnet, "Jsonnet");
    lang_names.insert(KnownProgLang::Ragel, "Ragel");
    lang_names.insert(KnownProgLang::Uno, "Uno");
    lang_names.insert(KnownProgLang::Hxml, "HXML");
    lang_names.insert(KnownProgLang::Hlsl, "HLSL");
    lang_names.insert(KnownProgLang::Ats, "ATS");
    lang_names.insert(KnownProgLang::Eiffel, "Eiffel");
    lang_names.insert(KnownProgLang::LinkerScript, "Linker Script");
    lang_names.insert(KnownProgLang::Tea, "Tea");
    lang_names.insert(KnownProgLang::Quake, "Quake");
    lang_names.insert(KnownProgLang::Kotlin, "Kotlin");
    lang_names.insert(KnownProgLang::Puppet, "Puppet");
    lang_names.insert(KnownProgLang::Vue, "Vue");
    lang_names.insert(KnownProgLang::Parrot, "Parrot");
    lang_names.insert(KnownProgLang::Ioke, "Ioke");
    lang_names.insert(KnownProgLang::Lua, "Lua");
    lang_names.insert(KnownProgLang::Sqf, "SQF");
    lang_names.insert(KnownProgLang::Mql4, "MQL4");
    lang_names.insert(KnownProgLang::Xml, "XML");
    lang_names.insert(KnownProgLang::Red, "Red");
    lang_names.insert(KnownProgLang::Moocode, "Moocode");
    lang_names.insert(KnownProgLang::Julia, "Julia");
    lang_names.insert(KnownProgLang::RawTokenData, "Raw token data");
    lang_names.insert(KnownProgLang::Smalltalk, "Smalltalk");
    lang_names.insert(KnownProgLang::M4Sugar, "M4Sugar");
    lang_names.insert(KnownProgLang::Zil, "ZIL");
    lang_names.insert(KnownProgLang::Mcfunction, "mcfunction");
    lang_names.insert(KnownProgLang::ColdFusionCfc, "ColdFusion CFC");
    lang_names.insert(KnownProgLang::AppleScript, "AppleScript");
    lang_names.insert(KnownProgLang::E, "E");
    lang_names.insert(KnownProgLang::Eq, "EQ");
    lang_names.insert(KnownProgLang::GroovyServerPages, "Groovy Server Pages");
    lang_names.insert(KnownProgLang::ObjDump, "ObjDump");
    lang_names.insert(KnownProgLang::Ruby, "Ruby");
    lang_names.insert(KnownProgLang::VisualBasicDotNet, "Visual Basic .NET");
    lang_names.insert(KnownProgLang::Thrift, "Thrift");
    lang_names.insert(KnownProgLang::IgorPro, "IGOR Pro");
    lang_names.insert(KnownProgLang::Asymptote, "Asymptote");
    lang_names.insert(KnownProgLang::Glsl, "GLSL");
    lang_names.insert(KnownProgLang::Nearly, "Nearly");
    lang_names.insert(KnownProgLang::SshConfig, "SSH Config");
    lang_names.insert(KnownProgLang::Shell, "Shell");
    lang_names.insert(KnownProgLang::Csv, "CSV");
    lang_names.insert(KnownProgLang::Edn, "edn");
    lang_names.insert(KnownProgLang::HtmlPlusErb, "HTML+ERB");
    lang_names.insert(KnownProgLang::Ceylon, "Ceylon");
    lang_names.insert(KnownProgLang::Lex, "Lex");
    lang_names.insert(KnownProgLang::CartoCss, "CartoCSS");
    lang_names.insert(KnownProgLang::EmberScript, "EmberScript");
    lang_names.insert(KnownProgLang::JsonLd, "JSONLD");
    lang_names.insert(KnownProgLang::Pickle, "Pickle");
    lang_names.insert(KnownProgLang::Prolog, "Prolog");
    lang_names.insert(KnownProgLang::TiProgram, "TI Program");
    lang_names.insert(KnownProgLang::AutoIt, "AutoIt");
    lang_names.insert(KnownProgLang::ObjectScript, "ObjectScript");
    lang_names.insert(KnownProgLang::CsoundScore, "Csound Score");
    lang_names.insert(KnownProgLang::Papyrus, "Papyrus");
    lang_names.insert(KnownProgLang::Turtle, "Turtle");
    lang_names.insert(KnownProgLang::Yara, "YARA");
    lang_names.insert(KnownProgLang::CapnProto, "Cap'n Proto");
    lang_names.insert(KnownProgLang::PostCss, "PostCSS");
    lang_names.insert(KnownProgLang::UrWeb, "UrWeb");
    lang_names.insert(KnownProgLang::Muse, "Muse");
    lang_names.insert(KnownProgLang::Muf, "MUF");
    lang_names.insert(KnownProgLang::AlpineAbuild, "Alpine Abuild");
    lang_names.insert(KnownProgLang::CObjDump, "C-ObjDump");
    lang_names.insert(KnownProgLang::Html, "HTML");
    lang_names.insert(KnownProgLang::Rust, "Rust");
    lang_names.insert(KnownProgLang::Frege, "Frege");
    lang_names.insert(KnownProgLang::IsabelleRoot, "Isabelle ROOT");
    lang_names.insert(
        KnownProgLang::WindowsRegistryEntries,
        "Windows Registry Entries",
    );
    lang_names.insert(KnownProgLang::Tcsh, "Tcsh");
    lang_names.insert(KnownProgLang::Racket, "Racket");
    lang_names.insert(KnownProgLang::Slim, "Slim");
    lang_names.insert(KnownProgLang::HtmlPlusPhp, "HTML+PHP");
    lang_names.insert(KnownProgLang::Fantom, "Fantom");
    lang_names.insert(KnownProgLang::JupyterNotebook, "Jupyter Notebook");
    lang_names.insert(KnownProgLang::Http, "HTTP");
    lang_names.insert(
        KnownProgLang::OpenStepPropertyList,
        "OpenStep Property List",
    );
    lang_names.insert(KnownProgLang::BlitzBasic, "BlitzBasic");
    lang_names.insert(KnownProgLang::Batchfile, "Batchfile");

    lang_names
});

impl Display for KnownProgLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            LANG_NAMES
                .get(self)
                .expect("all KnownProgLang entries should be present")
        )
    }
}

impl FromStr for KnownProgLang {
    type Err = EmbeddingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LANG_NAMES
            .iter()
            .find(|(_, name)| **name == s)
            .map(|(lang, _)| *lang)
            .ok_or_else(|| EmbeddingError::UnknownProgLang(s.clone_as_boxstr()))
    }
}
