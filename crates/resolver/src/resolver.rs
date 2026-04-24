use la_arena::Idx;
use rustc_hash::FxHashMap;
use sq_3_parser::{
    AstNode, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TextSize,
    ast::{
        self, ArrayLiteralExpression, BaseExpression, BinaryExpression, BinaryOperator,
        BlockStatement, BreakStatement, CallExpression, ClassExpression, ClassStatement,
        CloneExpression, ConditionalExpression, ConstStatement, ContinueStatement,
        DeleteExpression, DoWhileStatement, DocComment, DocTypeName, ElementAccessExpression,
        EnumStatement, Expr, ExpressionStatement, ExpressionWrapper, ForEachStatement,
        ForInitialiserKind, ForStatement, FunctionBody, FunctionExpression, FunctionStatement,
        HasBody, HasDescription, HasDoc, HasName, HasOperand, HasType, IfStatement, IsClass,
        IsClassMember, IsFunction, LambdaExpression, LiteralExpression, LiteralExpressionKind,
        LocalFunctionDeclaration, LocalVariableDeclaration, Member, MemberAccessExpression,
        MemberName, Name, Parameter, ParenthesisedExpression, PostfixUpdateExpression,
        PostfixUpdateOperator, PrefixUnaryExpression, PrefixUnaryOperator, PrefixUpdateExpression,
        PrefixUpdateOperator, Property, RawCallExpression, ResumeExpression, ReturnStatement,
        RootAccessExpression, SourceFile, Stmt, StringNameKind, SwitchClause, SwitchStatement,
        TableLiteralExpression, Tag, ThisExpression, ThrowStatement, TryStatement,
        TypeOfExpression, VariableDeclaration, WhileStatement, YieldStatement,
    },
};
use std::{collections::hash_map::Entry, path::PathBuf};

use crate::{
    Diagnostic, DiagnosticSeverity, ExpressionKind, File, FindSymbol, ImportMembers,
    NullableExprKind, Source, SourceSymbol, TypeWithRange,
    arena::{
        ArenaAlloc, ArenaId, ArrayData, ArrayId, ClassData, ClassId, Container, EnumData, EnumId,
        FunctionData, FunctionId, ImportTarget, ParamsState, Scope, ScopeId, SourceArena,
        StringLiteralData, StringLiteralId, SymbolId, TableData, TableId, TypeConversionError,
        UnionData, UnionId,
    },
    db::{Db, SpecialFunction},
    symbol::{
        LocalKind, PropertyKind, StringKind, Symbol, SymbolFlags, SymbolKind, SymbolTable, Type,
        TypeKind, TypeSet, TypeState, insert_symbol,
    },
};

macro_rules! dispatch_union {
    // For methods returning Option<T>
    ($self:ident, $operand:expr, $error_msg:expr, $single_method:ident $(, $extra:expr)*) => {{
        let operand = $operand;
        if let Type::Union(id) = operand.typ {
            let types = $self.get(id).types.clone();
            for typ in types {
                if let Some(result) = $self.$single_method(
                    TypeWithRange { typ, ..operand },
                    $($extra,)*
                    false,
                ) {
                    return Some(result);
                }
            }
            if !$self.get(id).type_set.contains(TypeSet::ANY) {
                $self.diagnostics.push(Diagnostic {
                    message: format!($error_msg, $self.type_to_str(operand.typ)),
                    range: operand.range,
                    severity: DiagnosticSeverity::Error,
                });
            }
            return None;
        }
        $self.$single_method(operand, $($extra,)* true)
    }};
}

#[derive(Debug, Clone)]
enum AssignmentLeftHandSide {
    CanCreate {
        parent: Type,
        expr_range: TextRange,
        new_key: Box<str>,
        name_range: TextRange,
    },
    // Parent doesn't exist for locals
    Exists {
        parent: Option<Type>,
        expr_range: TextRange,
        symbol: SymbolId,
        name_range: TextRange,
    },
    NonStringName {
        parent: Type,
        expr_range: TextRange,
        name: TypeWithRange,
    },
    Invalid(NullableExprKind),
}

const fn to_operand_and_arguments(
    parent: Type,
    expr_range: TextRange,
    name_range: TextRange,
    argument: TypeWithRange,
) -> (TypeWithRange, [TypeWithRange; 2]) {
    let operand = TypeWithRange {
        typ: parent,
        range: expr_range,
    };

    let arguments = [
        TypeWithRange {
            typ: Type::STRING,
            range: name_range,
        },
        argument,
    ];

    (operand, arguments)
}

fn lhs_container(lhs: Option<&AssignmentLeftHandSide>) -> Option<Container> {
    let parent = match lhs {
        Some(
            AssignmentLeftHandSide::CanCreate { parent, .. }
            | AssignmentLeftHandSide::NonStringName { parent, .. },
        ) => parent,
        Some(AssignmentLeftHandSide::Exists { parent, .. }) => parent.as_ref()?,
        Some(AssignmentLeftHandSide::Invalid(_)) | None => return None,
    };

    Container::try_from(*parent).ok()
}

impl From<&AssignmentLeftHandSide> for NullableExprKind {
    fn from(value: &AssignmentLeftHandSide) -> Self {
        match value {
            AssignmentLeftHandSide::Exists { symbol, .. } => Some(ExpressionKind::Symbol(*symbol)),
            AssignmentLeftHandSide::Invalid(key) => *key,
            AssignmentLeftHandSide::CanCreate { .. }
            | AssignmentLeftHandSide::NonStringName { .. } => None,
        }
    }
}

fn get_name<T>(node: &T) -> Option<SyntaxToken>
where
    T: HasName,
{
    let name = node.name()?;
    name.identifier()
}

impl TryFrom<AssignmentLeftHandSide> for Type {
    type Error = ();
    fn try_from(value: AssignmentLeftHandSide) -> Result<Self, Self::Error> {
        match value {
            AssignmentLeftHandSide::CanCreate { parent, .. }
            | AssignmentLeftHandSide::NonStringName { parent, .. } => Ok(parent),
            AssignmentLeftHandSide::Exists { parent, .. } => parent.ok_or(()),
            AssignmentLeftHandSide::Invalid(_) => Err(()),
        }
    }
}

impl TryFrom<AssignmentLeftHandSide> for Container {
    type Error = TypeConversionError;

    fn try_from(value: AssignmentLeftHandSide) -> Result<Self, Self::Error> {
        let typ = Type::try_from(value).map_err(|()| TypeConversionError::WrongType)?;
        Self::try_from(typ)
    }
}

#[derive(Debug, Clone, Copy)]
enum MetamethodErrors {
    No,
    Yes {
        keyword: &'static str,
    },
    YesBinary {
        keyword: &'static str,
        right: TypeWithRange,
    },
}

struct DeferredFunctionTrace {
    node: Box<dyn IsFunction>,
    scope: ScopeId,
}

struct DeferredFunctionEntry {
    idx: Idx<FunctionData>,
    trace: DeferredFunctionTrace,
}

#[derive(Debug, Clone, Copy)]
enum CheckTypeSource {
    Variable,
    Parameter,
    VarArgs,
    Return,
    Throw,
    Yield,
}

#[derive(Debug, Clone, Copy)]
enum NewSlotResult {
    CanAdd(Container),
    Allowed,
    NotAllowed,
}

#[derive(Debug, Clone, Copy)]
enum NewType {
    NotExplicit(TypeWithRange),
    Explicit { typ: Type, value_range: TextRange },
}

pub struct Resolver<'db> {
    db: &'db dyn Db,
    file: File,

    imports: FxHashMap<ImportTarget, Vec<File>>,

    arena: SourceArena,
    source_table: Idx<TableData>,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,

    scope: ScopeId,

    /// The container new members will be added to. Note that this is different from
    /// container that we take symbols from. That one is stored on the scope and can
    /// be acquired via .`execution_container()`
    container: Container,

    can_break: bool,
    can_continue: bool,
    dead_code: bool,

    function: Option<Idx<FunctionData>>,
    deferred_functions: FxHashMap<Idx<FunctionData>, DeferredFunctionTrace>,

    range_to_expr: FxHashMap<TextRange, ExpressionKind>,
    range_to_symbol: FxHashMap<TextRange, SymbolId>,
    doc_to_symbol: FxHashMap<TextRange, SymbolId>,
    symbol_to_ranges: FxHashMap<SymbolId, Vec<TextRange>>,
    diagnostics: Vec<Diagnostic>,
}

impl Source for Resolver<'_> {
    fn file(&self) -> File {
        self.file
    }

    fn db(&self) -> &dyn Db {
        self.db
    }

    fn arena(&self) -> &SourceArena {
        &self.arena
    }

    fn imports(&self) -> &FxHashMap<ImportTarget, Vec<File>> {
        &self.imports
    }

    fn scope(&self, _offset: TextSize) -> ScopeId {
        self.scope
    }

    fn source_table(&self) -> TableId {
        TableId::new(self.file, self.source_table)
    }

    fn root_table(&self) -> TableId {
        TableId::new(self.file, self.root_table)
    }

    fn const_table(&self) -> TableId {
        TableId::new(self.file, self.const_table)
    }

    fn range_to_expr(&self) -> &FxHashMap<TextRange, ExpressionKind> {
        &self.range_to_expr
    }

    fn range_to_symbol(&self) -> &FxHashMap<TextRange, SymbolId> {
        &self.range_to_symbol
    }

    fn doc_to_symbol(&self) -> &FxHashMap<TextRange, SymbolId> {
        &self.doc_to_symbol
    }

    fn symbol_to_ranges(&self) -> &FxHashMap<SymbolId, Vec<TextRange>> {
        &self.symbol_to_ranges
    }

    fn get<T>(&self, id: T) -> &T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>,
    {
        // To avoid cycle, get the data from the current file from here
        if id.file() != self.file {
            return id.get_data(self.db);
        }

        &self.arena[id.idx()]
    }

    fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl<'db> Resolver<'db> {
    pub fn symbol_from_source_file(db: &'db dyn Db, file: File, node: &SourceFile) -> SourceSymbol {
        let mut arena = SourceArena::default();
        // Source table is not always the root table, it depends on which entity
        // was the script executed. script_execute and non-edict entities execute stuff
        // in the root while edict entities with 'vscripts' keyvalue will have their
        // script scope as the execution context
        // This should also drive whether 'self' is present in the scope
        // TODO: Get source file's jsdoc and determine
        let source_table = arena.alloc(TableData::default());
        let container = Container::Table(TableId::new(file, source_table));
        let root_table = arena.alloc(TableData::default());
        let const_table = arena.alloc(TableData::default());
        let scope = arena.alloc(Scope {
            range: node.syntax().text_range(),
            ..Default::default()
        });

        let mut imports = FxHashMap::default();
        let mut libs = Vec::new();
        if let Some(squirrel_lib) = db.squirrel_lib()
            && squirrel_lib != file
        {
            libs.push(squirrel_lib);
        }

        if let Some(vscript_lib) = db.vscript_lib()
            && vscript_lib != file
        {
            libs.push(vscript_lib);
        }

        if !libs.is_empty() {
            imports.insert(ImportTarget::Table(TableId::new(file, root_table)), libs);
        }

        let mut collector = Self {
            db,
            file,
            imports,
            scope,
            container,
            can_break: false,
            can_continue: false,
            dead_code: false,
            arena,
            source_table,
            const_table,
            root_table,
            function: None,
            deferred_functions: FxHashMap::default(),
            range_to_expr: FxHashMap::default(),
            range_to_symbol: FxHashMap::default(),
            doc_to_symbol: FxHashMap::default(),
            symbol_to_ranges: FxHashMap::default(),
            diagnostics: Vec::new(),
        };

        let mut is_native = false;
        if let Some(doc) = node.doc() {
            for tag in doc.tags() {
                match tag {
                    Tag::Native(_) => is_native = true,
                    Tag::Entity(_) => {
                        let base_entity = db.base_entity_class();
                        let id = collector.symbol(Symbol {
                            name: "self".into(),
                            typ: Type::Instance(base_entity),
                            type_state: TypeState::Explicit,
                            kind: SymbolKind::Property(PropertyKind::Embedded),
                            name_range: tag.syntax().text_range(),
                            range: tag.syntax().text_range(),
                            ..Default::default()
                        });

                        collector.add_current_container_member("self".into(), id);
                    }
                    _ => {}
                }
            }
        }

        for stmt in node.statements() {
            collector.collect_stmt(&stmt);
        }

        assert_eq!(collector.arena[collector.scope].parent, None);

        // Resolve remaining functions
        while let Some(idx) = collector.deferred_functions.keys().next().copied() {
            let trace = collector
                .deferred_functions
                .remove(&idx)
                .expect("We just got this index");
            let entry = DeferredFunctionEntry { idx, trace };
            collector.resolve_function_doc(&entry, node.syntax().text_range().end());
            collector.resolve_deferred_function_entry(&entry);
        }

        if !is_native {
            collector.unused_variables_diagnostics();
        }

        collector.deprecated_diagnostics();

        SourceSymbol {
            imports: collector.imports,
            arena: collector.arena,
            const_table,
            root_table,
            source_table,
            range_to_expr: collector.range_to_expr,
            range_to_symbol: collector.range_to_symbol,
            doc_to_symbol: collector.doc_to_symbol,
            symbol_to_ranges: collector.symbol_to_ranges,
            diagnostics: collector.diagnostics,
        }
    }

    pub fn get_mut<T>(&mut self, id: T) -> Option<&mut T::Data>
    where
        T: ArenaId,
        SourceArena: std::ops::IndexMut<Idx<T::Data>, Output = T::Data>,
    {
        if id.file() != self.file {
            return None;
        }

        Some(&mut self.arena[id.idx()])
    }

    fn new_reference(&mut self, range: TextRange, id: SymbolId) {
        self.range_to_symbol.insert(range, id);
        self.symbol_to_ranges
            .entry(id)
            .and_modify(|list| list.push(range))
            .or_insert_with(|| vec![range]);
    }

    fn symbol(&mut self, symbol: Symbol) -> SymbolId {
        let name_range = symbol.name_range;
        let id = SymbolId::new(self.file, self.arena.alloc(symbol));
        self.new_reference(name_range, id);
        id
    }

    fn class(&mut self, class: &impl IsClass) -> ClassId {
        let expr = class.extends().and_then(|e| e.expression());

        let inherits = if let Some(expr) = expr {
            match self.expr_to_type(&expr) {
                Type::Class(id) => id,
                Type::Unknown | Type::Any => None,
                typ => {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to inherit from '{}'", self.type_to_str(typ)),
                        range: expr.syntax().text_range(),
                        ..Default::default()
                    });
                    None
                }
            }
        } else {
            None
        };

        ClassId::new(
            self.file,
            self.arena.alloc(ClassData {
                inherits,
                members: SymbolTable::default(),
                symbol: None,
            }),
        )
    }

    fn array(&mut self, array: ArrayData) -> ArrayId {
        ArrayId::new(self.file, self.arena.alloc(array))
    }

    fn string(&mut self, token_result: &(StringNameKind, SyntaxToken)) -> StringLiteralId {
        let (left_offset, right_offset, text) = match token_result.0 {
            StringNameKind::Normal => {
                let input = token_result.1.text();
                let (s, left) = input.strip_prefix('"').map_or((input, 0u32), |s| (s, 1));
                let (s, right) = s.strip_suffix('"').map_or((s, 0u32), |s| (s, 1));

                (left, right, s.to_owned())
            }
            StringNameKind::Verbatim => {
                let input = token_result.1.text();
                let (s, left) = input.strip_prefix("@\"").map_or((input, 0u32), |s| (s, 2));
                let (s, right) = s.strip_suffix('"').map_or((s, 0u32), |s| (s, 1));

                (
                    left,
                    right,
                    s.replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace("\"\"", "\\\""),
                )
            }
        };

        let range = token_result.1.text_range();

        let unquoted_range = TextRange::new(
            range.start() + TextSize::new(left_offset),
            range.end() - TextSize::new(right_offset),
        );

        StringLiteralId::new(
            self.file,
            self.arena.alloc(StringLiteralData {
                text: text.into_boxed_str(),
                range,
                unquoted_range,
            }),
        )
    }

    fn clone_type(&mut self, typ: Type) -> Type {
        match typ {
            Type::Table(Some(id)) => {
                let new = TableId::new(self.file, self.arena.alloc(self.get(id).clone()));
                Type::Table(Some(new))
            }
            Type::Class(Some(id)) => {
                let new = ClassId::new(self.file, self.arena.alloc(self.get(id).clone()));
                Type::Class(Some(new))
            }
            _ => typ,
        }
    }

    fn current_scope(&mut self) -> &mut Scope {
        &mut self.arena[self.scope]
    }

    fn enter_scope(&mut self, range: TextRange) {
        self.scope = self.arena.alloc(Scope {
            parent: Some(self.scope),
            locals: SymbolTable::default(),
            range,
            function: self.function,
        });
    }

    fn exit_scope(&mut self) {
        self.dead_code = false;
        self.scope = self.arena[self.scope]
            .parent
            .expect("We shouldn't use exit_scope without enter_scope first");
    }

    fn execution_container(&self) -> Container {
        self.function.map_or_else(
            || Container::Table(self.source_table()),
            |id| {
                let function = &self.arena[id];
                function.bindenv.unwrap_or(function.container)
            },
        )
    }

    fn add_current_container_member(&mut self, name: Box<str>, symbol: SymbolId) {
        self.add_container_member(self.container, name, symbol);
    }

    fn add_container_member(&mut self, container: Container, name: Box<str>, symbol: SymbolId) {
        match container {
            Container::Table(id) => {
                if let Some(t) = self.get_mut(id) {
                    insert_symbol(&mut t.members, name, symbol);
                }
            }
            Container::Class(id) | Container::Instance(id) => {
                if let Some(c) = self.get_mut(id) {
                    insert_symbol(&mut c.members, name, symbol);
                }
            }
            Container::Enum(id) => {
                if let Some(e) = self.get_mut(id) {
                    insert_symbol(&mut e.members, name, symbol);
                }
            }
        }
    }

    /// This is only a speculation, you can actually execute static member as an instance
    /// and the other way around. The best approximation is this though
    fn try_swap_to_instance(
        &mut self,
        member: &impl IsClassMember,
        method_id: Option<FunctionId>,
    ) -> PropertyKind {
        if let Container::Class(id) = self.container
            && member.static_keyword().is_none()
        {
            if let Some(func) = method_id.and_then(|id| self.get_mut(id)) {
                func.container = Container::Instance(id);
            }

            PropertyKind::No
        } else {
            PropertyKind::Yes
        }
    }

    fn no_member_error(&mut self, obj: Type, member_name: &str, error_range: TextRange) {
        if TypeSet::CAN_HAVE_UNKNOWN_MEMBERS.contains(self.to_type_set(obj)) {
            return;
        }

        self.diagnostics.push(Diagnostic {
            message: match obj {
                Type::Enum(id) => {
                    let Some(name) = self.get(id).symbol.map(|s| &self.get(s).name) else {
                        return;
                    };

                    format!("enum '{name}' has no member named '{member_name}'")
                }
                _ => format!(
                    "'{}' has no member named '{}'",
                    self.type_to_str(obj),
                    member_name
                ),
            },
            range: error_range,
            ..Default::default()
        });
    }

    fn resolve_name(&self, text: &str, offset: TextSize) -> Option<SymbolId> {
        let filter = |(name, id): (Box<str>, SymbolId)| {
            if name.as_ref() == text {
                Some(id)
            } else {
                None
            }
        };

        let locals = self.local_members(offset).into_iter().find_map(filter);

        let consts = || {
            self.members_of_table(
                self.const_table(),
                FindSymbol::OnlyBefore(offset),
                ImportMembers::Const,
            )
            .into_iter()
            .find_map(filter)
        };

        let members = || {
            self.members_of_container(
                self.execution_container(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(filter)
        };

        let root = || {
            self.members_of_table(
                self.root_table(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                ImportMembers::Root,
            )
            .into_iter()
            .find_map(filter)
        };

        locals.or_else(consts).or_else(members).or_else(root)
    }

    fn doc_type_single(&mut self, name: &DocTypeName, offset: TextSize) -> Option<Type> {
        let identifier = name.identifier()?;
        let text = identifier.text();
        let typ = match text {
            "any" => Type::Any,
            "int" | "integer" => Type::INTEGER,
            "float" => Type::FLOAT,
            "string" => Type::STRING,
            "bool" | "boolean" => Type::BOOLEAN,
            "null" => Type::Null,
            "instance" => Type::INSTANCE,
            "array" => Type::ARRAY,
            "table" => Type::TABLE,
            "class" => Type::CLASS,
            "function" => Type::FUNCTION,
            "generator" => Type::GENERATOR,
            "thread" => Type::THREAD,
            "weakref" => Type::Weakref,
            _ => {
                if let Ok(kind) = text.parse::<StringKind>() {
                    Type::String {
                        kind,
                        literal: None,
                    }
                } else {
                    let Some(id) = self.resolve_name(text, offset) else {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "Couldn't find type '{identifier}', defaulting to using 'unknown''"
                            ),
                            range: name.syntax().text_range(),
                            severity: DiagnosticSeverity::Information,
                        });
                        return None;
                    };

                    let Type::Class(id) = self.get(id).typ else {
                        return None;
                    };

                    Type::Instance(id)
                }
            }
        };

        if let Some(symbol) = self.type_to_symbol(typ) {
            self.new_reference(name.syntax().text_range(), symbol);
        }

        Some(typ)
    }

    fn doc_type(
        &mut self,
        mut types: impl Iterator<Item = DocTypeName>,
        offset: TextSize,
    ) -> Option<Type> {
        let first = types.next()?;

        let mut last_type = self.doc_type_single(&first, offset);
        for typ in types {
            let Some(next_type) = self.doc_type_single(&typ, offset) else {
                continue;
            };

            if let Some(typ) = last_type {
                last_type = Some(self.merge_or_union(typ, next_type));
            } else {
                last_type = Some(next_type);
            }
        }

        last_type
    }

    fn merge_types(&self, left: Type, right: Type) -> Option<Type> {
        if left == right {
            return Some(left);
        }

        Some(match (left, right) {
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Integer(_), Type::Integer(_)) => Type::INTEGER,
            (Type::Float(_), Type::Float(_)) => Type::FLOAT,
            (Type::Boolean(_), Type::Boolean(_)) => Type::BOOLEAN,
            (Type::String { .. }, Type::String { .. }) => Type::STRING,

            (Type::Instance(Some(left_id)), Type::Instance(Some(right_id))) => {
                if left_id == right_id {
                    return Some(left);
                }

                let mut class_id = left_id;
                while let Some(inherits) = self.get(class_id).inherits {
                    if inherits == right_id {
                        return Some(Type::Instance(Some(inherits)));
                    }
                    class_id = inherits;
                }

                class_id = right_id;
                while let Some(inherits) = self.get(class_id).inherits {
                    if inherits == left_id {
                        return Some(Type::Instance(Some(inherits)));
                    }
                    class_id = inherits;
                }
                return None;
            }
            (Type::Instance(_), Type::Instance(_)) => Type::INSTANCE,
            (Type::Table(_), Type::Table(_)) => Type::TABLE,
            (Type::Class(_), Type::Class(_)) => Type::CLASS,
            (Type::Array(_), Type::Array(_)) => Type::ARRAY,
            (Type::Function(_), Type::Function(_)) => Type::FUNCTION,
            (Type::Generator(_), Type::Generator(_)) => Type::GENERATOR,
            (Type::Thread(_), Type::Thread(_)) => Type::THREAD,
            (Type::Union(_), _) | (_, Type::Union(_)) => {
                panic!("Union type should not be passed into 'merge_types'")
            }
            (_, _) => {
                return None;
            }
        })
    }

    fn merge_or_union(&mut self, left: Type, right: Type) -> Type {
        match (left, right) {
            (Type::Union(left_id), Type::Union(right_id)) => {
                let mut result = Vec::new();
                let mut right_used = vec![false; self.get(right_id).types.len()];

                let left_types = self.get(left_id).types.clone();
                let right_types = self.get(right_id).types.clone();

                for left in left_types {
                    let mut merged = false;

                    for (i, right) in right_types.iter().enumerate() {
                        if right_used[i] {
                            continue;
                        }

                        if let Some(new_type) = self.merge_types(left, *right) {
                            result.push(new_type);
                            right_used[i] = true;
                            merged = true;
                            break;
                        }
                    }

                    if !merged {
                        result.push(left);
                    }
                }

                // Add remaining right-side types
                for (i, right) in right_types.iter().enumerate() {
                    if !right_used[i] {
                        result.push(*right);
                    }
                }

                Type::Union(UnionId::new(
                    self.file,
                    self.arena.alloc(UnionData {
                        types: result,
                        type_set: self
                            .get(left_id)
                            .type_set
                            .union(self.get(right_id).type_set),
                    }),
                ))
            }

            (other, Type::Union(union_id)) | (Type::Union(union_id), other) => {
                let mut types = Vec::new();
                let mut iter = self.get(union_id).types.iter();
                let type_set = self
                    .get(union_id)
                    .type_set
                    .union(TypeSet::from_kind(other.into()));

                while let Some(typ) = iter.next() {
                    let Some(merged_type) = self.merge_types(*typ, other) else {
                        types.push(*typ);
                        continue;
                    };

                    types.push(merged_type);
                    // After we've successfully merged the required type just extend the list
                    // with the remaining types from the iterator
                    types.extend(iter);
                    return Type::Union(UnionId::new(
                        self.file,
                        self.arena.alloc(UnionData { type_set, types }),
                    ));
                }
                // No merge was successful -> just add a new type to the end of the list
                types.push(other);
                Type::Union(UnionId::new(
                    self.file,
                    self.arena.alloc(UnionData { type_set, types }),
                ))
            }
            (left, right) => {
                if let Some(typ) = self.merge_types(left, right) {
                    return typ;
                }

                let types = vec![left, right];
                let type_set = TypeSet::new(&[left.into(), right.into()]);

                Type::Union(UnionId::new(
                    self.file,
                    self.arena.alloc(UnionData { type_set, types }),
                ))
            }
        }
    }

    fn is_type_suitable(&self, left: Type, right: Type) -> bool {
        match (left, right) {
            (Type::Float(_), Type::Integer(_))
            | (Type::Any | Type::Unknown, _)
            | (_, Type::Any | Type::Unknown) => true,

            (Type::Instance(Some(left_id)), Type::Instance(Some(right_id))) => {
                if left_id == right_id {
                    return true;
                }

                let mut class_id = left_id;
                while let Some(inherits) = self.get(class_id).inherits {
                    if inherits == right_id {
                        return true;
                    }
                    class_id = inherits;
                }

                class_id = right_id;
                while let Some(inherits) = self.get(class_id).inherits {
                    if inherits == left_id {
                        return true;
                    }
                    class_id = inherits;
                }
                false
            }

            (Type::Union(left_id), Type::Union(right_id)) => {
                let right_types = &self.get(right_id).types;
                for left_type in &self.get(left_id).types {
                    for right_type in right_types {
                        if self.is_type_suitable(*left_type, *right_type) {
                            return true;
                        }
                    }
                }

                false
            }

            (other, Type::Union(union_id)) | (Type::Union(union_id), other) => {
                for typ in &self.get(union_id).types {
                    if self.is_type_suitable(*typ, other) {
                        return true;
                    }
                }

                false
            }

            (_, _) => TypeKind::from(left) == right.into(),
        }
    }

    fn check_string_literal(
        &mut self,
        left: Type,
        right: Type,
        error_range: TextRange,
        can_modify_left: bool,
    ) -> Option<Type> {
        let Type::String { kind, .. } = left else {
            return None;
        };

        let Type::String {
            literal: Some(literal),
            ..
        } = right
        else {
            return None;
        };

        let text = &self.get(literal).text;

        let text = if kind.is_case_sensetive() {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        let message = match kind {
            StringKind::Script => self.db().get_script(PathBuf::from(text)).err(),
            _ => {
                if kind
                    .values()
                    .is_some_and(|values| !values.iter().any(|set| set.0.contains(&text)))
                {
                    Some(format!(
                        "Text of string literal is not suitable for the kind '{kind}'"
                    ))
                } else {
                    None
                }
            }
        };

        if let Some(message) = message {
            self.diagnostics.push(Diagnostic {
                message,
                range: error_range,
                severity: DiagnosticSeverity::Warning,
            });
        }

        if !can_modify_left {
            return None;
        }

        let typ = Type::String {
            kind,
            literal: Some(literal),
        };

        if literal.file() == self.file {
            self.range_to_expr
                .insert(self.get(literal).range, ExpressionKind::Literal(typ));
        }

        Some(typ)
    }

    fn check_type(
        &mut self,
        left: Type,
        right: Type,
        source: CheckTypeSource,
        error_range: TextRange,
        can_modify_left: bool,
    ) -> Option<Type> {
        if self.is_type_suitable(left, right) {
            return self.check_string_literal(left, right, error_range, can_modify_left);
        }

        self.diagnostics.push(Diagnostic {
            message: match source {
                CheckTypeSource::Variable => format!(
                    "Trying to assign a variable of type '{}' to '{}'",
                    self.type_to_str(left),
                    self.type_to_str(right)
                ),
                CheckTypeSource::VarArgs |
                CheckTypeSource::Parameter => format!(
                    "Expected parameter of type '{}', but got '{}'",
                    self.type_to_str(left),
                    self.type_to_str(right)
                ),
                CheckTypeSource::Return => format!(
                    "Trying to return a value of type '{}' in a function with declared return type of '{}'",
                    self.type_to_str(left),
                    self.type_to_str(right),
                ),
                CheckTypeSource::Throw => format!(
                    "Trying to throw a value of type '{}' in a function with declared throw type of '{}'",
                    self.type_to_str(left),
                    self.type_to_str(right),
                ),
                CheckTypeSource::Yield => format!(
                    "Trying to yield a value of type '{}' in a function with declared yield type of '{}'",
                    self.type_to_str(left),
                    self.type_to_str(right),
                ),
            },
            range: error_range,
            severity: DiagnosticSeverity::Warning,
        });

        None
    }

    fn check_or_update_type(
        &mut self,
        current: Type,
        current_state: TypeState,
        new: NewType,
        source: CheckTypeSource,
    ) -> Option<Type> {
        match current_state {
            TypeState::NotAssigned => Some(match new {
                NewType::Explicit { typ, .. } => typ,
                NewType::NotExplicit(new) => new.typ,
            }),
            TypeState::Inferred => match new {
                NewType::NotExplicit(new) => Some(self.merge_or_union(current, new.typ)),
                NewType::Explicit { typ, value_range } => {
                    self.check_type(typ, current, source, value_range, true)
                }
            },
            TypeState::Explicit => match new {
                NewType::Explicit { typ, .. } => Some(typ),
                NewType::NotExplicit(new) => {
                    self.check_type(current, new.typ, source, new.range, false)
                }
            },
        }
    }

    fn collect_params(
        &mut self,
        idx: Idx<FunctionData>,
        parameters: impl Iterator<Item = Parameter>,
    ) {
        let mut params_state = ParamsState::NoDefault;

        for (count, param) in parameters.enumerate() {
            match param {
                Parameter::Variable(var) => {
                    let Some(name) = get_name(&var) else {
                        let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                            continue;
                        };

                        self.collect_expr(&expr);
                        continue;
                    };

                    let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                        match params_state {
                            ParamsState::Default(_) => {
                                self.diagnostics.push(Diagnostic {
                                    message: "Non-default parameter cannot be preceded by a default parameter".to_owned(),
                                    range: var.syntax().text_range(),
                                    ..Default::default()
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::VarArgs(_, _) => {
                                self.diagnostics.push(Diagnostic {
                                    message: "Parameters cannot be preceded by varied arguments"
                                        .to_owned(),
                                    range: var.syntax().text_range(),
                                    ..Default::default()
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::NoDefault => {}
                        }

                        let symbol = self.symbol(Symbol {
                            name: name.text().into(),
                            typ: Type::Unknown,
                            kind: SymbolKind::Local(LocalKind::Parameter),
                            name_range: name.text_range(),
                            range: var.syntax().text_range(),
                            ..Default::default()
                        });

                        self.resolve_variable_doc(symbol, &var);

                        insert_symbol(&mut self.current_scope().locals, name.text().into(), symbol);
                        self.arena[idx].params.push(symbol);
                        continue;
                    };

                    let typ = self.expr_to_type(&expr);

                    let symbol = self.symbol(Symbol {
                        name: name.text().into(),
                        typ,
                        kind: SymbolKind::Local(LocalKind::Parameter),
                        name_range: name.text_range(),
                        range: var.syntax().text_range(),
                        ..Default::default()
                    });

                    self.resolve_variable_doc(symbol, &var);

                    insert_symbol(&mut self.current_scope().locals, name.text().into(), symbol);
                    self.arena[idx].params.push(symbol);

                    match params_state {
                        ParamsState::NoDefault => {
                            params_state = ParamsState::Default(count);
                        }
                        ParamsState::Default(_) => {}
                        ParamsState::VarArgs(var_args_at, _) => {
                            self.diagnostics.push(Diagnostic {
                                message: "Parameters cannot be preceded by varied arguments"
                                    .to_owned(),
                                range: var.syntax().text_range(),
                                ..Default::default()
                            });
                            params_state = ParamsState::Default(var_args_at);
                        }
                    }
                }
                Parameter::Ellipsis(var_args) => match params_state {
                    ParamsState::NoDefault => {
                        let vargv_array = self.array(ArrayData { typ: Type::Unknown });
                        let symbol = self.symbol(Symbol {
                            name: "vargv".into(),
                            typ: Type::Array(Some(vargv_array)),
                            kind: SymbolKind::Local(LocalKind::VariedArgs),
                            name_range: var_args.syntax().text_range(),
                            range: var_args.syntax().text_range(),
                            ..Default::default()
                        });

                        insert_symbol(&mut self.current_scope().locals, "vargv".into(), symbol);
                        params_state = ParamsState::VarArgs(count, symbol);
                    }
                    ParamsState::Default(_) => {
                        self.diagnostics.push(Diagnostic {
                            message:
                                "Function with varied arguments cannot have default parameters"
                                    .to_owned(),
                            range: var_args.syntax().text_range(),
                            ..Default::default()
                        });
                    }
                    ParamsState::VarArgs(_, _) => {
                        self.diagnostics.push(Diagnostic {
                            message: "There can't be 2 varied arguments in a function signature"
                                .to_owned(),
                            range: var_args.syntax().text_range(),
                            ..Default::default()
                        });
                    }
                },
            }
        }

        self.arena[idx].params_state = params_state;
    }

    fn call_metamethod(
        &mut self,
        callable: TypeWithRange,
        metamethod: &str,
        arguments: &[TypeWithRange],
        errors: MetamethodErrors,
    ) -> Option<Type> {
        match callable.typ {
            Type::Table(id) => {
                let Some(id) = id else {
                    return Some(Type::Unknown);
                };

                let table = self.get(id);
                let Some(delegate_idx) = table.delegate else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "'table' does not support {keyword}: no delegate assigned"
                                ),
                                range: callable.range,
                                ..Default::default()
                            });
                        }
                        MetamethodErrors::No => {}
                    }
                    return None;
                };

                // possibly change error_range.start() to the real offset parameter?
                let Some(member) = self.find_member(
                    Container::Table(delegate_idx),
                    metamethod,
                    callable.range.start(),
                ) else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!("'table' does not support {keyword}: delegate has no '{metamethod}' metamethod"),
                                range: callable.range,
                                ..Default::default()
                            });
                        }
                        MetamethodErrors::No => {}
                    }
                    return None;
                };

                Some(self.callable(
                    callable.typ,
                    TypeWithRange {
                        typ: member.typ,
                        range: callable.range,
                    },
                    arguments,
                )?)
            }
            Type::Instance(id) => {
                let Some(id) = id else {
                    return Some(Type::Unknown);
                };

                let Some(member) =
                    self.find_member(Container::Instance(id), metamethod, callable.range.start())
                else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!("'instance' does not support {keyword}: class has no '{metamethod}' metamethod"),
                                range: callable.range,
                                ..Default::default()
                            });
                        }
                        MetamethodErrors::No => {}
                    }
                    return None;
                };

                Some(self.callable(
                    callable.typ,
                    TypeWithRange {
                        typ: member.typ,
                        range: callable.range,
                    },
                    arguments,
                )?)
            }
            Type::Unknown | Type::Any => None,
            _ => {
                match errors {
                    MetamethodErrors::Yes { keyword } => {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "'{}' does not support {}",
                                self.type_to_str(callable.typ),
                                keyword
                            ),
                            range: callable.range,
                            ..Default::default()
                        });
                    }
                    MetamethodErrors::YesBinary { keyword, right } => {
                        if !matches!(right.typ, Type::Unknown | Type::Any) {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "'{}' does not support {} with '{}'",
                                    self.type_to_str(callable.typ),
                                    keyword,
                                    self.type_to_str(right.typ)
                                ),
                                range: right.range,
                                ..Default::default()
                            });
                        }
                    }
                    MetamethodErrors::No => {}
                }
                None
            }
        }
    }

    fn new_slot_single(
        &mut self,
        operand: TypeWithRange,
        arguments: &[TypeWithRange],
        should_error: bool,
    ) -> NewSlotResult {
        match operand.typ {
            Type::Class(id) => {
                let Some(id) = id else {
                    return NewSlotResult::Allowed;
                };
                NewSlotResult::CanAdd(Container::Class(id))
            }
            Type::Table(id) => {
                let Some(id) = id else {
                    return NewSlotResult::Allowed;
                };
                self.call_metamethod(operand, "_newslot", arguments, MetamethodErrors::No);
                NewSlotResult::CanAdd(Container::Table(id))
            }
            _ => {
                if self
                    .call_metamethod(
                        operand,
                        "_newslot",
                        arguments,
                        if should_error {
                            MetamethodErrors::Yes {
                                keyword: "new slot operator",
                            }
                        } else {
                            MetamethodErrors::No
                        },
                    )
                    .is_none()
                {
                    return NewSlotResult::NotAllowed;
                }

                NewSlotResult::CanAdd(Container::try_from(operand.typ).expect(
                    "Type that did not fail `_newslot` metamethod call has to be a container",
                ))
            }
        }
    }

    fn new_slot(&mut self, operand: TypeWithRange, arguments: &[TypeWithRange]) -> NewSlotResult {
        if let Type::Union(id) = operand.typ {
            let types = self.get(id).types.clone();
            for typ in types {
                if matches!(
                    self.new_slot_single(TypeWithRange { typ, ..operand }, arguments, false),
                    NewSlotResult::NotAllowed
                ) {
                    continue;
                }

                return NewSlotResult::Allowed;
            }
            if self.get(id).type_set.contains(TypeSet::ANY) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "'{}' does not support new slot operator",
                        self.type_to_str(operand.typ)
                    ),
                    range: operand.range,
                    severity: DiagnosticSeverity::Error,
                });
            }
            return NewSlotResult::NotAllowed;
        }
        self.new_slot_single(operand, arguments, true)
    }

    fn delete_single(
        &mut self,
        operand: TypeWithRange,
        index: TypeWithRange,
        should_error: bool,
    ) -> Option<Type> {
        match operand.typ {
            Type::Class(_) => Some(Type::Unknown),
            Type::Table(_) => {
                self.call_metamethod(operand, "_delslot", &[index], MetamethodErrors::No)
            }
            _ => self.call_metamethod(
                operand,
                "_delslot",
                &[index],
                if should_error {
                    MetamethodErrors::Yes {
                        keyword: "delete operator",
                    }
                } else {
                    MetamethodErrors::No
                },
            ),
        }
    }

    fn delete(&mut self, operand: TypeWithRange, index: TypeWithRange) -> Option<Type> {
        dispatch_union!(
            self,
            operand,
            "'{}' does not support equals operator",
            delete_single,
            index
        )
    }

    fn set_single(
        &mut self,
        operand: TypeWithRange,
        arguments: &[TypeWithRange],
        should_error: bool,
    ) -> Option<Type> {
        match operand.typ {
            Type::Array(_) | Type::Class(_) => Some(arguments.last()?.typ),
            Type::Table(_) | Type::Instance(_) => Some(
                self.call_metamethod(operand, "_set", arguments, MetamethodErrors::No)
                    .unwrap_or(arguments.last()?.typ),
            ),
            _ => self.call_metamethod(
                operand,
                "_set",
                arguments,
                if should_error {
                    MetamethodErrors::Yes {
                        keyword: "equals operator",
                    }
                } else {
                    MetamethodErrors::No
                },
            ),
        }
    }

    fn set(&mut self, operand: TypeWithRange, arguments: &[TypeWithRange]) -> Option<Type> {
        dispatch_union!(
            self,
            operand,
            "'{}' does not support equals operator",
            set_single,
            arguments
        )
    }

    fn arithmetic_single(
        &mut self,
        operand: TypeWithRange,
        with: TypeWithRange,
        operator: BinaryOperator,
        should_error: bool,
    ) -> Option<Type> {
        let (metamethod, keyword) = match operator {
            BinaryOperator::Add | BinaryOperator::AddAssign => ("_add", "adding"),
            BinaryOperator::Subtract | BinaryOperator::SubtractAssign => ("_sub", "subtracting"),
            BinaryOperator::Multiply | BinaryOperator::MultiplyAssign => ("_mul", "multiplying"),
            BinaryOperator::Divide | BinaryOperator::DivideAssign => ("_div", "dividing"),
            BinaryOperator::Modulo | BinaryOperator::ModuloAssign => ("_modulo", "modulo"),
            _ => unreachable!(),
        };

        let operand_set = self.to_type_set(operand.typ);
        let with_set = self.to_type_set(with.typ);

        if (operator == BinaryOperator::Add || operator == BinaryOperator::AddAssign)
            && (TypeSet::STRING.contains(operand_set) || TypeSet::STRING.contains(with_set))
        {
            return Some(Type::STRING);
        }

        if TypeSet::INTEGER.contains(operand_set) && TypeSet::INTEGER.contains(with_set) {
            return Some(Type::INTEGER);
        }

        if TypeSet::are_both_numbers(operand_set, with_set) {
            return Some(Type::FLOAT);
        }

        self.call_metamethod(
            operand,
            metamethod,
            &[with],
            if should_error {
                MetamethodErrors::YesBinary {
                    keyword,
                    right: with,
                }
            } else {
                MetamethodErrors::No
            },
        )
    }

    fn arithmetic(
        &mut self,
        operand: TypeWithRange,
        with: TypeWithRange,
        operator: BinaryOperator,
    ) -> Option<Type> {
        dispatch_union!(
            self,
            operand,
            "'{}' does not support arithmetic operations",
            arithmetic_single,
            with,
            operator
        )
    }

    fn iterable_single(
        &mut self,
        iterable: TypeWithRange,
        should_error: bool,
    ) -> Option<(Type, Type)> {
        match iterable.typ {
            Type::Table(_) => {
                let arguments = [TypeWithRange {
                    typ: Type::Null,
                    ..iterable
                }];
                self.call_metamethod(iterable, "_nexti", &arguments, MetamethodErrors::No);
                Some((Type::Unknown, Type::Unknown))
            }
            Type::Array(id) => {
                let typ = id.map_or(Type::Unknown, |id| self.get(id).typ);
                Some((Type::INTEGER, typ))
            }
            Type::Generator(id) => {
                let typ = id.map_or(Type::Unknown, |id| self.get(id).yields);

                Some((Type::INTEGER, typ))
            }
            Type::Class(_) => Some((Type::Unknown, Type::Unknown)),
            _ => {
                let arguments = [TypeWithRange {
                    typ: Type::Null,
                    ..iterable
                }];

                self.call_metamethod(
                    iterable,
                    "_nexti",
                    &arguments,
                    if should_error {
                        MetamethodErrors::Yes {
                            keyword: "iterating",
                        }
                    } else {
                        MetamethodErrors::No
                    },
                )
                .map(|typ| (Type::Unknown, typ))
            }
        }
    }

    fn iterable(&mut self, iterable: TypeWithRange) -> Option<(Type, Type)> {
        dispatch_union!(
            self,
            iterable,
            "'{}' does not support iterating",
            iterable_single
        )
    }

    fn callable_single(
        &mut self,
        callable: TypeWithRange,
        context: Type,
        arguments: &[TypeWithRange],
        should_error: bool,
    ) -> Option<Type> {
        match callable.typ {
            Type::Function(id) => {
                let Some(id) = id else {
                    return Some(Type::Unknown);
                };

                let data = self.deferred_entry(id);
                if let Some(ref data) = data {
                    self.resolve_function_doc(data, callable.range.end());
                }

                for (count, argument) in arguments.iter().copied().enumerate() {
                    let Some(&param) = self.get(id).params.get(count) else {
                        let ParamsState::VarArgs(_, vargv) = self.get(id).params_state else {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "Passing {} parameters when only {} is possible",
                                    count + 1,
                                    self.get(id).params.len()
                                ),
                                range: argument.range,
                                ..Default::default()
                            });
                            continue;
                        };

                        let typ = self.get(vargv).typ;

                        let Type::Array(Some(id)) = typ else {
                            continue;
                        };

                        let Some(new_typ) = self.check_or_update_type(
                            typ,
                            self.get(vargv).type_state,
                            NewType::NotExplicit(argument),
                            CheckTypeSource::VarArgs,
                        ) else {
                            continue;
                        };

                        if let Some(symbol) = self.get_mut(vargv) {
                            symbol.type_state = TypeState::Inferred;
                        }

                        if let Some(array) = self.get_mut(id) {
                            array.typ = new_typ;
                        }
                        continue;
                    };

                    let Some(new) = self.check_or_update_type(
                        self.get(param).typ,
                        self.get(param).type_state,
                        NewType::NotExplicit(argument),
                        CheckTypeSource::Parameter,
                    ) else {
                        continue;
                    };

                    if let Some(param) = self.get_mut(param) {
                        param.typ = new;
                        param.type_state = TypeState::Inferred;
                    }
                }

                let least_params_required = match self.get(id).params_state {
                    ParamsState::NoDefault => self.get(id).params.len(),
                    ParamsState::Default(from) | ParamsState::VarArgs(from, _) => from,
                };

                if arguments.len() < least_params_required {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "Passing {} parameters when at least {} is required",
                            arguments.len(),
                            least_params_required
                        ),
                        range: callable.range,
                        ..Default::default()
                    });
                }

                // We resolve the params first so we can get param type substitution before we run the body
                // However since we reuse the result, the function body can have errors that it wouldn't have
                // if we get the new type information? Probably not since function body must complete
                // with the exact type the user has passed in
                if let Some(data) = data {
                    self.resolve_deferred_function_entry(&data);
                }

                match self.db.check_native(id) {
                    Some(SpecialFunction::IncludeScript | SpecialFunction::DoIncludeScript) => {
                        self.include_script(arguments);
                    }
                    Some(SpecialFunction::GetRootTable) => {
                        // Overrides return
                        return Some(Type::Table(Some(self.root_table())));
                    }
                    Some(SpecialFunction::GetConstTable) => {
                        // Overrides return
                        return Some(Type::Table(Some(self.const_table())));
                    }
                    Some(SpecialFunction::NewThread) => {
                        if let Some(first) = arguments.first()
                            && let Type::Function(func) = first.typ
                        {
                            return Some(Type::Thread(func));
                        }
                        return Some(Type::THREAD);
                    }
                    Some(SpecialFunction::SetDelegate) => {
                        self.set_delegate(context, arguments);
                        return Some(context);
                    }
                    Some(SpecialFunction::Bindenv) => {
                        return Some(self.bindenv(context, arguments));
                    }
                    None => {}
                }

                Some(if self.get(id).yields_state == TypeState::NotAssigned {
                    self.clone_type(self.get(id).ret)
                } else {
                    Type::Generator(Some(id))
                })
            }
            Type::Class(id) => {
                if let Some(symbol) =
                    self.find_member(Container::Class(id?), "constructor", callable.range.start())
                {
                    self.callable(
                        context,
                        TypeWithRange {
                            typ: symbol.typ,
                            ..callable
                        },
                        arguments,
                    );
                } else if !arguments.is_empty() {
                    self.diagnostics.push(Diagnostic {
                        message: "Default constructor should have no parameters".to_owned(),
                        range: callable.range,
                        ..Default::default()
                    });
                }

                Some(Type::Instance(id))
            }
            _ => self.call_metamethod(
                callable,
                "_call",
                arguments,
                if should_error {
                    MetamethodErrors::Yes { keyword: "calling" }
                } else {
                    MetamethodErrors::No
                },
            ),
        }
    }

    fn callable(
        &mut self,
        context: Type,
        callable: TypeWithRange,
        arguments: &[TypeWithRange],
    ) -> Option<Type> {
        dispatch_union!(
            self,
            callable,
            "'{}' does not support calling",
            callable_single,
            context,
            arguments
        )
    }

    fn check_constant(&mut self, expr: NullableExprKind, range: TextRange) {
        match expr {
            Some(ExpressionKind::Literal(
                Type::Integer(Some(_))
                | Type::Float(Some(_))
                | Type::Boolean(Some(_))
                | Type::String {
                    literal: Some(_), ..
                },
            )) => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message:
                        "Constant can only hold value of 'integer', 'float', 'string' or 'bool'"
                            .to_owned(),
                    range,
                    ..Default::default()
                });
            }
        }
    }

    fn collect_function<T>(&mut self, node: &T) -> FunctionId
    where
        T: IsFunction + Clone + 'static,
    {
        let bindenv = node
            .environment()
            .and_then(|e| e.expression())
            .map(|env| (env.syntax().text_range(), self.expr_to_type(&env)))
            .and_then(|(range, typ)| {
                if let Ok(container) = Container::try_from(typ) {
                    Some(container)
                } else {
                    if !matches!(typ, Type::Unknown | Type::Any) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "Trying to use '{}' as function's environment",
                                self.type_to_str(typ)
                            ),
                            range,
                            severity: DiagnosticSeverity::Warning,
                        });
                    }
                    None
                }
            });

        let range = match node.body() {
            Some(FunctionBody::Expr(body)) => body.syntax().text_range(),
            Some(FunctionBody::Stmt(body)) => body.syntax().text_range(),
            None => TextRange::empty(node.syntax().text_range().end()),
        };

        let id = FunctionId::new(
            self.file,
            self.arena.alloc(FunctionData {
                symbol: None,
                range,
                container: self.container,
                bindenv,
                params: Vec::new(),
                params_state: ParamsState::NoDefault,
                ret: Type::Unknown,
                ret_state: TypeState::NotAssigned,
                throws: Type::Unknown,
                throws_state: TypeState::NotAssigned,
                yields: Type::Unknown,
                yields_state: TypeState::NotAssigned,
            }),
        );

        self.enter_scope(range);

        if let Some(param_list) = node.parameter_list() {
            self.collect_params(id.idx(), param_list.parameters());
        }

        self.deferred_functions.insert(
            id.idx(),
            DeferredFunctionTrace {
                node: Box::new(node.clone()),
                scope: self.scope,
            },
        );

        self.exit_scope();

        id
    }

    fn resolve_variable_doc<T>(&mut self, symbol: SymbolId, node: &T) -> bool
    where
        T: HasDoc,
    {
        let Some(doc) = node.doc() else {
            return false;
        };

        let range = doc.syntax().text_range();
        match self.doc_to_symbol.entry(range) {
            Entry::Occupied(_) => return true,
            Entry::Vacant(e) => {
                e.insert(symbol);
            }
        }

        let offset = range.end();
        for tag in doc.tags() {
            match tag {
                Tag::Type(type_tag) => {
                    let Some(typ) = type_tag.typ() else {
                        continue;
                    };

                    let Some(doc_type) = self.doc_type(typ.types(), offset) else {
                        continue;
                    };

                    if let Some(typ) = self.check_or_update_type(
                        self.get(symbol).typ,
                        self.get(symbol).type_state,
                        NewType::Explicit {
                            typ: doc_type,
                            value_range: self.get(symbol).range,
                        },
                        CheckTypeSource::Variable,
                    ) && let Some(symbol) = self.get_mut(symbol)
                    {
                        symbol.typ = typ;
                        symbol.type_state = TypeState::Explicit;
                    }
                }
                Tag::Const(_) => {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.flags |= SymbolFlags::CONST;
                    }
                }
                Tag::Hide(_) => {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.flags |= SymbolFlags::HIDE;
                    }
                }
                Tag::Deprecated(_) => {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.flags |= SymbolFlags::DEPRECATED;
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn resolve_function_doc(&mut self, entry: &DeferredFunctionEntry, offset: TextSize) {
        let Some(doc) = entry.trace.node.doc().or_else(|| {
            let syntax = entry.trace.node.syntax();
            if !matches!(
                syntax.kind(),
                SyntaxKind::FunctionExpression | SyntaxKind::LambdaExpression
            ) {
                return None;
            }
            parent_doc(syntax)
        }) else {
            return;
        };

        if let Some(symbol_id) = self.arena[entry.idx].symbol
            && let Some(symbol) = self.get_mut(symbol_id)
            && let Some(desc) = doc.description()
        {
            symbol.description = desc.content();
        }

        for tag in doc.tags() {
            match tag {
                Tag::Return(tag) => {
                    let Some(typ) = tag.typ() else {
                        continue;
                    };

                    if let Some(doc_type) = self.doc_type(typ.types(), offset) {
                        self.arena[entry.idx].ret = doc_type;
                        self.arena[entry.idx].ret_state = TypeState::Explicit;
                    }
                }
                Tag::Param(tag) => {
                    let Some(param_name) = tag.name().and_then(|n| n.identifier()) else {
                        continue;
                    };
                    let text = param_name.text();

                    let Some(param_id) = self.arena[entry.idx]
                        .params
                        .iter()
                        .rev()
                        .find(|id| self.get(**id).name.as_ref() == text)
                        .copied()
                    else {
                        self.diagnostics.push(Diagnostic {
                            message: format!("Couldn't find param '{text}'"),
                            range: param_name.text_range(),
                            severity: DiagnosticSeverity::Information,
                        });
                        continue;
                    };

                    self.new_reference(param_name.text_range(), param_id);

                    if let Some(param) = self.get_mut(param_id)
                        && let Some(desc) = tag.description()
                    {
                        param.description = desc.content();
                    }

                    let Some(typ) = tag.typ() else {
                        continue;
                    };

                    let Some(doc_type) = self.doc_type(typ.types(), offset) else {
                        continue;
                    };

                    // Default parameters are resolved in 'collect_function' since they're evaluated immediately
                    // after seeing it, and since the doc comment is resolved here we match the type of the default
                    // parameter with our annotated type
                    let current_type = self.get(param_id).typ;
                    if !self.is_type_suitable(doc_type, current_type) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "Trying to assign a default value of type '{}' to a variable of type '{}'",
                                self.type_to_str(current_type),
                                self.type_to_str(doc_type)
                            ),
                            range: self.get(param_id).range,
                            severity: DiagnosticSeverity::Warning
                        });
                    }

                    if let Some(param) = self.get_mut(param_id) {
                        param.typ = doc_type;
                        param.type_state = TypeState::Explicit;
                    }
                }
                Tag::Throw(tag) => {
                    let Some(typ) = tag.typ() else {
                        continue;
                    };

                    if let Some(doc_type) = self.doc_type(typ.types(), offset) {
                        self.arena[entry.idx].throws = doc_type;
                        self.arena[entry.idx].throws_state = TypeState::Explicit;
                    }
                }
                Tag::Yield(tag) => {
                    let Some(typ) = tag.typ() else {
                        continue;
                    };

                    if let Some(doc_type) = self.doc_type(typ.types(), offset) {
                        self.arena[entry.idx].yields = doc_type;
                        self.arena[entry.idx].yields_state = TypeState::Explicit;
                    }
                }
                Tag::VarArgs(tag) => {
                    let ParamsState::VarArgs(_, id) = self.arena[entry.idx].params_state else {
                        continue;
                    };

                    if let Some(symbol) = self.get_mut(id)
                        && let Some(desc) = tag.description()
                    {
                        symbol.description = desc.content();
                    }

                    let Some(typ) = tag.typ() else {
                        continue;
                    };

                    let Some(typ) = self.doc_type(typ.types(), offset) else {
                        continue;
                    };

                    let array_id = self.array(ArrayData { typ });
                    if let Some(symbol) = self.get_mut(id) {
                        symbol.typ = Type::Array(Some(array_id));
                        symbol.type_state = TypeState::Explicit;
                    }
                }
                _ => {}
            }
        }
    }

    fn resolve_deferred_function_entry(&mut self, entry: &DeferredFunctionEntry) {
        // No reason to change stuff if function has no body
        let Some(body) = entry.trace.node.body() else {
            return;
        };

        let function = &self.arena[entry.idx];

        let save_container = self.container;
        self.container = function.bindenv.unwrap_or(function.container);
        let save_scope = self.scope;
        self.scope = entry.trace.scope;
        let save_function = self.function;
        self.function = Some(entry.idx);
        let save_dead_code = self.dead_code;
        self.dead_code = false;
        let save_break = self.can_break;
        self.can_break = false;
        let save_continue = self.can_continue;
        self.can_continue = false;

        match body {
            FunctionBody::Expr(expr) => {
                let new_ret = self.expr_to_type_with_range(&expr);

                if let Some(new) = self.check_or_update_type(
                    self.arena[entry.idx].ret,
                    self.arena[entry.idx].ret_state,
                    NewType::NotExplicit(new_ret),
                    CheckTypeSource::Return,
                ) {
                    self.arena[entry.idx].ret = new;
                    self.arena[entry.idx].ret_state = TypeState::Inferred;
                }
            }
            FunctionBody::Stmt(stmt) => {
                self.collect_stmt(&stmt);

                if self.arena[entry.idx].ret_state == TypeState::NotAssigned {
                    self.arena[entry.idx].ret = Type::Null;
                    self.arena[entry.idx].ret_state = TypeState::Inferred;
                }
            }
        }

        self.container = save_container;
        self.scope = save_scope;
        self.function = save_function;
        self.dead_code = save_dead_code;
        self.can_break = save_break;
        self.can_continue = save_continue;
    }

    fn deferred_entry(&mut self, id: FunctionId) -> Option<DeferredFunctionEntry> {
        // If function is external it is already resolved
        if id.file() != self.file {
            return None;
        }

        let idx = id.idx();
        // If function is not in deferred_functions it is already resolved
        let trace = self.deferred_functions.remove(&idx)?;

        Some(DeferredFunctionEntry { idx, trace })
    }

    fn get_member_name(&mut self, name: MemberName) -> Option<(TextRange, Box<str>)> {
        match name {
            MemberName::Identifier(n) => {
                let name = n.name()?;
                let identifier = name.identifier()?;
                Some((identifier.text_range(), identifier.text().into()))
            }
            MemberName::String(n) => {
                let id = n.token().map(|r| self.string(&r))?;
                let s = self.get(id);
                Some((s.unquoted_range, s.text.clone()))
            }
            MemberName::Computed(n) => {
                let kind = self.collect_expr(&n.expression()?);
                let Some(ExpressionKind::Literal(Type::String {
                    literal: Some(literal),
                    ..
                })) = kind
                else {
                    return None;
                };
                let s = self.get(literal);
                Some((s.unquoted_range, s.text.clone()))
            }
        }
    }

    fn collect_table_member(&mut self, member: &Member) {
        match member {
            Member::Property(property) => self.collect_table_property(property),
            Member::Method(method) => {
                let id = self.collect_function(method);

                let Some(name) = get_name(method) else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: name.text().into(),
                    typ: Type::Function(Some(id)),
                    name_range: name.text_range(),
                    range: method.syntax().text_range(),
                    ..Default::default()
                });

                if let Some(function) = self.get_mut(id) {
                    function.symbol = Some(symbol);
                }

                self.resolve_variable_doc(symbol, method);

                self.add_current_container_member(name.text().into(), symbol);
            }
            Member::Constructor(constructor) => {
                let id = self.collect_function(constructor);

                let Some(keyword) = constructor.constructor_keyword() else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: "constructor".into(),
                    typ: Type::Function(Some(id)),
                    name_range: keyword.text_range(),
                    range: constructor.syntax().text_range(),
                    ..Default::default()
                });

                if let Some(function) = self.get_mut(id) {
                    function.symbol = Some(symbol);
                }

                self.resolve_variable_doc(symbol, constructor);

                self.add_current_container_member("constructor".into(), symbol);
            }
        }
    }

    fn collect_class_member(&mut self, member: &Member) {
        match member {
            Member::Property(property) => self.collect_class_property(property),
            Member::Method(method) => {
                let id = self.collect_function(method);
                let statik = self.try_swap_to_instance(method, Some(id));

                let Some(name) = get_name(method) else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: name.text().into(),
                    typ: Type::Function(Some(id)),
                    kind: SymbolKind::Property(statik),
                    name_range: name.text_range(),
                    range: method.syntax().text_range(),
                    ..Default::default()
                });

                if let Some(function) = self.get_mut(id) {
                    function.symbol = Some(symbol);
                }

                self.resolve_variable_doc(symbol, method);

                self.add_current_container_member(name.text().into(), symbol);
            }
            Member::Constructor(constructor) => {
                let id = self.collect_function(constructor);
                let statik = self.try_swap_to_instance(constructor, Some(id));

                let Some(keyword) = constructor.constructor_keyword() else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: "constructor".into(),
                    typ: Type::Function(Some(id)),
                    kind: SymbolKind::Property(statik),
                    name_range: keyword.text_range(),
                    range: constructor.syntax().text_range(),
                    ..Default::default()
                });

                if let Some(function) = self.get_mut(id) {
                    function.symbol = Some(symbol);
                }

                self.resolve_variable_doc(symbol, constructor);

                self.add_current_container_member("constructor".into(), symbol);
            }
        }
    }

    fn collect_table_property(&mut self, property: &Property) {
        let typ = property
            .value()
            .map_or(Type::Unknown, |v| self.expr_to_type(&v));

        let Some(name) = property.name() else {
            return;
        };

        let Some((name_range, text)) = self.get_member_name(name) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ,
            name_range,
            range: property.syntax().text_range(),
            type_state: if typ == Type::Null {
                TypeState::NotAssigned
            } else {
                TypeState::Inferred
            },
            ..Default::default()
        });

        self.resolve_variable_doc(symbol, property);

        self.add_current_container_member(text, symbol);
    }

    fn collect_class_property(&mut self, property: &Property) {
        let typ = property
            .value()
            .map_or(Type::Unknown, |v| self.expr_to_type(&v));

        let statik = self.try_swap_to_instance(
            property,
            match typ {
                Type::Function(id) => id,
                _ => None,
            },
        );

        let Some(name) = property.name() else {
            return;
        };

        let Some((name_range, text)) = self.get_member_name(name) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ,
            kind: SymbolKind::Property(statik),
            name_range,
            range: property.syntax().text_range(),
            type_state: if typ == Type::Null {
                TypeState::NotAssigned
            } else {
                TypeState::Inferred
            },
            ..Default::default()
        });

        self.resolve_variable_doc(symbol, property);

        self.add_current_container_member(text, symbol);
    }

    /// returns whether the value was assigned via '=' (used to increment the internal auto assign counter)
    fn collect_enum_property(&mut self, property: &Property, default_value: i32) -> bool {
        let (has_value, typ) =
            property
                .value()
                .map_or((false, Type::Integer(Some(default_value))), |expr| {
                    let value = self.collect_expr(&expr);
                    self.check_constant(value, expr.syntax().text_range());
                    (true, self.expr_kind_to_type(value))
                });

        let Some(name) = property.name() else {
            return has_value;
        };

        let Some((name_range, text)) = self.get_member_name(name) else {
            return has_value;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ,
            kind: SymbolKind::EnumMember,
            name_range,
            range: property.syntax().text_range(),
            type_state: TypeState::Inferred,
            ..Default::default()
        });

        self.resolve_variable_doc(symbol, property);

        self.add_current_container_member(text, symbol);
        has_value
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        if self.dead_code && !matches!(stmt, Stmt::Empty(_)) {
            self.diagnostics.push(Diagnostic {
                message: "Unreachable statement".to_owned(),
                range: stmt.syntax().text_range(),
                severity: DiagnosticSeverity::Unnecessary,
            });
        }

        match stmt {
            Stmt::LocalVariable(stmt) => self.local_variable(stmt),
            Stmt::LocalFunction(stmt) => self.local_function(stmt),
            Stmt::Block(stmt) => self.block_statement(stmt),
            Stmt::Const(stmt) => self.const_statement(stmt),
            Stmt::ForEach(stmt) => self.for_each_statement(stmt),
            Stmt::For(stmt) => self.for_statement(stmt),
            Stmt::Class(stmt) => self.class_statement(stmt),
            Stmt::Function(stmt) => self.function_statement(stmt),
            Stmt::Enum(stmt) => self.enum_statement(stmt),
            Stmt::Expression(stmt) => self.expression_statement(stmt),
            Stmt::Empty(_) => (),
            Stmt::If(stmt) => self.if_statement(stmt),
            Stmt::While(stmt) => self.while_statement(stmt),
            Stmt::DoWhile(stmt) => self.do_while_statement(stmt),
            Stmt::Switch(stmt) => self.switch_statement(stmt),
            Stmt::Return(stmt) => self.return_statement(stmt),
            Stmt::Yield(stmt) => self.yield_statement(stmt),
            Stmt::Continue(stmt) => self.continue_statement(stmt),
            Stmt::Break(stmt) => self.break_statement(stmt),
            Stmt::Try(stmt) => self.try_statement(stmt),
            Stmt::Throw(stmt) => self.throw_statement(stmt),
        }
    }

    fn local_variable(&mut self, decl: &LocalVariableDeclaration) {
        for var in decl.declarations() {
            let Some(name) = get_name(&var) else {
                let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                    continue;
                };

                self.collect_expr(&expr);
                continue;
            };

            let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                let id = self.symbol(Symbol {
                    name: name.text().into(),
                    typ: Type::Null,
                    kind: SymbolKind::Local(LocalKind::Variable),
                    name_range: name.text_range(),
                    range: var.syntax().text_range(),
                    ..Default::default()
                });

                if !self.resolve_variable_doc(id, &var) {
                    self.resolve_variable_doc(id, decl);
                }

                insert_symbol(&mut self.current_scope().locals, name.text().into(), id);
                continue;
            };

            let typ = self.expr_to_type(&expr);
            let id = self.symbol(Symbol {
                name: name.text().into(),
                typ,
                type_state: TypeState::Inferred,
                kind: SymbolKind::Local(LocalKind::Variable),
                name_range: name.text_range(),
                range: var.syntax().text_range(),
                ..Default::default()
            });

            if !self.resolve_variable_doc(id, &var) {
                self.resolve_variable_doc(id, decl);
            }

            insert_symbol(&mut self.current_scope().locals, name.text().into(), id);
        }
    }

    fn local_function(&mut self, decl: &LocalFunctionDeclaration) {
        let id = self.collect_function(decl);
        let Some(name) = get_name(decl) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: name.text().into(),
            typ: Type::Function(Some(id)),
            kind: SymbolKind::Local(LocalKind::Function),
            name_range: name.text_range(),
            range: decl.syntax().text_range(),
            ..Default::default()
        });

        if let Some(function) = self.get_mut(id) {
            function.symbol = Some(symbol);
        }

        self.resolve_variable_doc(symbol, decl);

        insert_symbol(&mut self.current_scope().locals, name.text().into(), symbol);
    }

    fn block_statement(&mut self, stmt: &BlockStatement) {
        self.enter_scope(stmt.syntax().text_range());
        for stmt in stmt.statements() {
            self.collect_stmt(&stmt);
        }
        self.exit_scope();
    }

    fn const_statement(&mut self, stmt: &ConstStatement) {
        let typ = stmt
            .value()
            .and_then(|v| v.expression())
            .map_or(Type::Unknown, |expr| {
                let value = self.collect_expr(&expr);
                self.check_constant(value, expr.syntax().text_range());
                self.expr_kind_to_type(value)
            });

        let Some(name) = get_name(stmt) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: name.text().into(),
            typ,
            kind: SymbolKind::Constant,
            name_range: name.text_range(),
            range: stmt.syntax().text_range(),
            type_state: TypeState::Inferred,
            ..Default::default()
        });

        self.resolve_variable_doc(symbol, stmt);

        insert_symbol(
            &mut self.arena[self.const_table].members,
            name.text().into(),
            symbol,
        );
    }

    fn for_each_statement(&mut self, stmt: &ForEachStatement) {
        let save_break_continue = (self.can_break, self.can_continue);
        self.can_break = true;
        self.can_continue = true;
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
        } else {
            self.enter_scope(TextRange::empty(stmt.syntax().text_range().end()));
        }

        let (key_type, value_type) =
            stmt.iterable()
                .map_or((Type::Unknown, Type::Unknown), |iterable| {
                    let typ = self.expr_to_type_with_range(&iterable);
                    self.iterable(typ).unwrap_or((Type::Unknown, Type::Unknown))
                });

        if let Some(key) = stmt.key()
            && let Some(name) = get_name(&key)
        {
            let symbol = self.symbol(Symbol {
                name: name.text().into(),
                typ: key_type,
                kind: SymbolKind::Local(LocalKind::Variable),
                name_range: name.text_range(),
                range: key.syntax().text_range(),
                ..Default::default()
            });

            insert_symbol(&mut self.current_scope().locals, name.text().into(), symbol);
        }

        if let Some(value) = stmt.value()
            && let Some(name) = get_name(&value)
        {
            let symbol = self.symbol(Symbol {
                name: name.text().into(),
                typ: value_type,
                kind: SymbolKind::Local(LocalKind::Variable),
                name_range: name.text_range(),
                range: value.syntax().text_range(),
                ..Default::default()
            });

            insert_symbol(&mut self.current_scope().locals, name.text().into(), symbol);
        }

        if let Some(body) = stmt.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
        (self.can_break, self.can_continue) = save_break_continue;
    }

    fn for_statement(&mut self, stmt: &ForStatement) {
        let save_break_continue = (self.can_break, self.can_continue);
        self.can_break = true;
        self.can_continue = true;
        self.enter_scope(stmt.syntax().text_range());
        match stmt.initialiser().and_then(|i| i.kind()) {
            Some(ForInitialiserKind::LocalVariableDeclaration(decl)) => self.local_variable(&decl),
            Some(ForInitialiserKind::LocalFunctionDeclaration(decl)) => self.local_function(&decl),
            Some(ForInitialiserKind::Expression(expr)) => {
                self.collect_expr(&expr);
            }
            None => {}
        }
        if let Some(condition) = stmt.condition().and_then(|c| c.expression()) {
            self.collect_expr(&condition);
        }
        if let Some(increment) = stmt.increment().and_then(|i| i.expression()) {
            self.collect_expr(&increment);
        }
        if let Some(body) = stmt.body() {
            self.collect_stmt(&body);
        }
        self.exit_scope();
        (self.can_break, self.can_continue) = save_break_continue;
    }

    fn class_statement(&mut self, stmt: &ClassStatement) {
        let class = self.class(stmt);

        let name = stmt.name().and_then(|n| self.assignment_lhs(&n));
        if let Some(symbol) = self.do_new_slot(
            None,
            name,
            TypeWithRange {
                typ: Type::Class(Some(class)),
                range: stmt.syntax().text_range(),
            },
            PropertyKind::NoSupport,
        ) {
            self.resolve_variable_doc(symbol, stmt);
        }

        let save_symbol = self.container;
        self.container = Container::Class(class);
        for member in stmt.members() {
            self.collect_class_member(&member);
        }
        self.container = save_symbol;
    }

    fn function_statement(&mut self, stmt: &FunctionStatement) {
        let id = self.collect_function(stmt);

        let Some(qualified_name) = stmt.name() else {
            return;
        };

        let mut parts = qualified_name.parts();

        let Some(first) = parts.next().and_then(|p| get_name(&p)) else {
            let Some(name) = get_name(&qualified_name) else {
                return;
            };

            // Plain `function abc()`: declare in current container
            let symbol = self.symbol(Symbol {
                name: name.text().into(),
                typ: Type::Function(Some(id)),
                name_range: name.text_range(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            });

            if let Some(function) = self.get_mut(id) {
                function.symbol = Some(symbol);
            }

            self.resolve_variable_doc(symbol, stmt);

            self.add_current_container_member(name.text().into(), symbol);
            return;
        };

        let text = first.text();

        let offset = qualified_name.syntax().text_range().end();

        let members = self
            .members_of_container(
                self.execution_container(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(|(name, id)| {
                if text == name.as_ref() {
                    Some(id)
                } else {
                    None
                }
            });

        let root = || {
            self.members_of_table(
                self.root_table(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                ImportMembers::Root,
            )
            .into_iter()
            .find_map(|(name, id)| {
                if text == name.as_ref() {
                    Some(id)
                } else {
                    None
                }
            })
        };

        let range = first.text_range();
        let Some(symbol_id) = members.or_else(root) else {
            if self
                .local_members(offset)
                .into_iter()
                .any(|(name, _)| text == name.as_ref())
            {
                self.diagnostics.push(Diagnostic {
                    message: "Function statement does not lookup locals. Initial symbol not found"
                        .to_owned(),
                    range,
                    severity: DiagnosticSeverity::Information,
                });
            }
            return;
        };

        let mut typ = TypeWithRange {
            typ: self.get(symbol_id).typ,
            range,
        };
        self.new_reference(range, symbol_id);

        for segment in parts {
            let arguments = [
                TypeWithRange {
                    typ: Type::STRING,
                    range: typ.range,
                },
                TypeWithRange {
                    typ: Type::Unknown,
                    range: segment.syntax().text_range(),
                },
            ];

            let NewSlotResult::CanAdd(container) = self.new_slot(typ, &arguments) else {
                return;
            };

            let Some(name_token) = get_name(&segment) else {
                return;
            };

            let Some(id) = self
                .members_of_container(
                    container,
                    FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                    false,
                )
                .into_iter()
                .find_map(|(name, id)| {
                    if name.as_ref() == name_token.text() {
                        Some(id)
                    } else {
                        None
                    }
                })
            else {
                return;
            };

            let range = name_token.text_range();
            typ = TypeWithRange {
                typ: self.get(id).typ,
                range,
            };
            self.new_reference(range, id);
        }

        let Some(final_name) = get_name(&qualified_name) else {
            return;
        };

        let arguments = [
            TypeWithRange {
                typ: Type::STRING,
                range: typ.range,
            },
            TypeWithRange {
                typ: Type::Function(Some(id)),
                range: final_name.text_range(),
            },
        ];

        let NewSlotResult::CanAdd(container) = self.new_slot(typ, &arguments) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: final_name.text().into(),
            typ: Type::Function(Some(id)),
            name_range: final_name.text_range(),
            range: stmt.syntax().text_range(),
            ..Default::default()
        });

        if let Some(function) = self.get_mut(id) {
            function.symbol = Some(symbol);
            function.container = container;
        }

        self.resolve_variable_doc(symbol, stmt);

        self.add_container_member(container, final_name.text().into(), symbol);
    }

    fn enum_statement(&mut self, stmt: &EnumStatement) {
        let enum_ = EnumId::new(self.file, self.arena.alloc(EnumData::default()));

        if let Some(name) = get_name(stmt) {
            let symbol = self.symbol(Symbol {
                name: name.text().into(),
                typ: Type::Enum(enum_),
                kind: SymbolKind::Enum,
                name_range: name.text_range(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            });

            self.resolve_variable_doc(symbol, stmt);

            self.arena[enum_.idx()].symbol = Some(symbol);

            insert_symbol(
                &mut self.arena[self.const_table].members,
                name.text().into(),
                symbol,
            );
        }

        let save_symbol = self.container;
        self.container = Container::Enum(enum_);
        let mut value = 0;
        for property in stmt.members() {
            if !self.collect_enum_property(&property, value) {
                value += 1;
            }
        }
        self.container = save_symbol;
    }

    fn expression_statement(&mut self, stmt: &ExpressionStatement) {
        let Some(expr) = stmt.expression() else {
            return;
        };

        self.collect_expr(&expr);
    }

    fn if_statement(&mut self, stmt: &IfStatement) {
        if let Some(condition) = stmt.condition() {
            self.collect_expr(&condition);
        }

        if let Some(then_stmt) = stmt.statement() {
            self.enter_scope(then_stmt.syntax().text_range());
            self.collect_stmt(&then_stmt);
            self.exit_scope();
        }

        if let Some(else_stmt) = stmt.else_branch().and_then(|e| e.statement()) {
            self.enter_scope(else_stmt.syntax().text_range());
            self.collect_stmt(&else_stmt);
            self.exit_scope();
        }
    }

    fn while_statement(&mut self, stmt: &WhileStatement) {
        if let Some(condition) = stmt.condition() {
            self.collect_expr(&condition);
        }

        if let Some(body) = stmt.body() {
            let save_break_continue = (self.can_break, self.can_continue);
            self.can_break = true;
            self.can_continue = true;
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
            (self.can_break, self.can_continue) = save_break_continue;
        }
    }

    fn do_while_statement(&mut self, stmt: &DoWhileStatement) {
        if let Some(body) = stmt.body() {
            let save_break_continue = (self.can_break, self.can_continue);
            self.can_break = true;
            self.can_continue = true;
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
            (self.can_break, self.can_continue) = save_break_continue;
        }

        if let Some(condition) = stmt.condition() {
            self.collect_expr(&condition);
        }
    }

    fn switch_statement(&mut self, stmt: &SwitchStatement) {
        let typ = if let Some(discriminant) = stmt.discriminant() {
            let disc = self.expr_to_type_with_range(&discriminant);
            let set = self.to_type_set(disc.typ);
            // Possibly use
            // pub const fn fully_contains(self, other: TypeSet) -> bool {
            //     (self.0 & other.0) == other.o
            // }
            // Instead
            if !TypeSet::VALID_SWITCH_DISCRIMINANT.contains(set) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "Discriminant of type '{}' depends on the variable addresses",
                        self.type_to_str(disc.typ)
                    ),
                    range: disc.range,
                    severity: DiagnosticSeverity::Warning,
                });
            }
            Some(disc.typ)
        } else {
            None
        };

        let save_break = self.can_break;
        self.can_break = true;
        for clause in stmt.clauses() {
            match clause {
                SwitchClause::Case(case) => {
                    if let Some(test) = case.test() {
                        let case_type = self.expr_to_type_with_range(&test);
                        if let Some(discriminant_typ) = typ {
                            let case_set = self.to_type_set(case_type.typ);
                            let discriminant_set = self.to_type_set(discriminant_typ);
                            if !discriminant_set.contains(case_set)
                                && !TypeSet::are_both_numbers(case_set, discriminant_set)
                            {
                                self.diagnostics.push(Diagnostic {
                                    message: format!("Case of type '{}' is incompitable with the discriminant of type '{}'", self.type_to_str(case_type.typ), self.type_to_str(discriminant_typ)),
                                    range: case_type.range,
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                        }
                    }

                    self.enter_scope(case.syntax().text_range());
                    for stmt in case.body() {
                        self.collect_stmt(&stmt);
                    }
                    self.exit_scope();
                }
                SwitchClause::Default(default) => {
                    self.enter_scope(default.syntax().text_range());
                    for stmt in default.body() {
                        self.collect_stmt(&stmt);
                    }
                    self.exit_scope();
                }
            }
        }
        self.can_break = save_break;
    }

    fn return_statement(&mut self, stmt: &ReturnStatement) {
        let value = stmt.value().map_or_else(
            || TypeWithRange::at_node(stmt.syntax()),
            |v| self.expr_to_type_with_range(&v),
        );

        self.dead_code = true;

        let Some(function) = self.function else {
            if stmt.value().is_some() {
                self.diagnostics.push(Diagnostic {
                    message: "Value returned by the source file execution scope cannot be received in any way".to_owned(),
                    range: stmt.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
            }
            return;
        };

        if let Some(new) = self.check_or_update_type(
            self.arena[function].ret,
            self.arena[function].ret_state,
            NewType::NotExplicit(value),
            CheckTypeSource::Return,
        ) {
            self.arena[function].ret = new;
            self.arena[function].ret_state = TypeState::Inferred;
        }
    }

    fn yield_statement(&mut self, stmt: &YieldStatement) {
        let value = stmt.value().map_or_else(
            || TypeWithRange::at_node(stmt.syntax()),
            |v| self.expr_to_type_with_range(&v),
        );

        let Some(function) = self.function else {
            self.diagnostics.push(Diagnostic {
                message: "Yielding in the source file execution scope".to_owned(),
                range: stmt.syntax().text_range(),
                severity: DiagnosticSeverity::Warning,
            });
            return;
        };

        if let Some(new) = self.check_or_update_type(
            self.arena[function].yields,
            self.arena[function].yields_state,
            NewType::NotExplicit(value),
            CheckTypeSource::Yield,
        ) {
            self.arena[function].yields = new;
            self.arena[function].yields_state = TypeState::Inferred;
        }
    }

    fn continue_statement(&mut self, stmt: &ContinueStatement) {
        if !self.can_continue {
            self.diagnostics.push(Diagnostic {
                message: "'continue' has to be in a loop block".to_owned(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            });
        }
        self.dead_code = true;
    }

    fn break_statement(&mut self, stmt: &BreakStatement) {
        if !self.can_break {
            self.diagnostics.push(Diagnostic {
                message: "'break' has to be in a loop or 'switch' block".to_owned(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            });
        }
        self.dead_code = true;
    }

    fn try_statement(&mut self, stmt: &TryStatement) {
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
        }

        let Some(catch) = stmt.catch_clause() else {
            return;
        };

        self.enter_scope(catch.body().map_or_else(
            || TextRange::empty(catch.syntax().text_range().end()),
            |body| body.syntax().text_range(),
        ));

        if let Some(binding) = catch.binding()
            && let Some(name) = get_name(&binding)
        {
            let symbol = self.symbol(Symbol {
                typ: Type::STRING,
                name: name.text().into(),
                kind: SymbolKind::Local(LocalKind::Exception),
                name_range: name.text_range(),
                range: binding.syntax().text_range(),
                ..Default::default()
            });

            self.resolve_variable_doc(symbol, &binding);

            insert_symbol(&mut self.current_scope().locals, name.text().into(), symbol);
        }

        if let Some(body) = catch.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
    }

    fn throw_statement(&mut self, stmt: &ThrowStatement) {
        // mark current function as exception throwing
        let typ = stmt
            .value()
            .map_or(Type::Unknown, |v| self.expr_to_type(&v));

        self.dead_code = true;
        let Some(function) = self.function else {
            return;
        };

        if let Some(new) = self.check_or_update_type(
            self.arena[function].throws,
            self.arena[function].throws_state,
            NewType::NotExplicit(TypeWithRange {
                typ,
                range: stmt.syntax().text_range(),
            }),
            CheckTypeSource::Throw,
        ) {
            self.arena[function].throws = new;
            self.arena[function].throws_state = TypeState::Inferred;
        }
    }

    fn expr_to_type(&mut self, expr: &Expr) -> Type {
        let kind = self.collect_expr(expr);
        self.expr_kind_to_type(kind)
    }

    fn expr_to_type_with_range(&mut self, expr: &Expr) -> TypeWithRange {
        TypeWithRange {
            typ: self.expr_to_type(expr),
            range: expr.syntax().text_range(),
        }
    }

    fn collect_expr(&mut self, expr: &Expr) -> NullableExprKind {
        let kind = match expr {
            Expr::Literal(expr) => self.literal_expression(expr),
            Expr::TableLiteral(expr) => Some(self.table_literal_expression(expr)),
            Expr::Class(expr) => Some(self.class_expression(expr)),
            Expr::ArrayLiteral(expr) => Some(self.array_literal_expression(expr)),
            Expr::Name(expr) => self.name_expression(expr),
            Expr::This(expr) => Some(self.this_expression(expr)),
            Expr::RootAccess(expr) => self.root_access_expression(expr),
            Expr::Base(expr) => Some(self.base_expression(expr)),
            Expr::MemberAccess(expr) => self.member_access_expression(expr),
            Expr::ElementAccess(expr) => self.element_access_expression(expr),
            Expr::Call(expr) => self.call_expression(expr),
            Expr::Clone(expr) => self.clone_expression(expr),
            Expr::Binary(expr) => self.binary_expression(expr),
            Expr::Conditional(expr) => Some(self.conditional_expression(expr)),
            Expr::PrefixUnary(expr) => self.prefix_unary_expression(expr),
            Expr::PrefixUpdate(expr) => self.prefix_update_expression(expr),
            Expr::PostfixUpdate(expr) => self.postfix_update_expression(expr),
            Expr::Delete(expr) => self.delete_expression(expr),
            Expr::TypeOf(expr) => Some(self.type_of_expression(expr)),
            Expr::Resume(expr) => self.resume_expression(expr),
            Expr::RawCall(expr) => self.raw_call_expression(expr),
            Expr::File(_) => Some(ExpressionKind::Literal(Type::STRING)),
            Expr::Line(_) => Some(ExpressionKind::Literal(Type::INTEGER)),
            Expr::Parenthesised(expr) => self.parenthesised_expression(expr),
            Expr::Function(expr) => Some(self.function_expression(expr)),
            Expr::Lambda(expr) => Some(self.lambda_expression(expr)),
        };

        if let Some(kind) = kind {
            let range = expr.syntax().text_range();
            self.range_to_expr.insert(range, kind);
        }

        kind
    }

    fn literal_expression(&mut self, expr: &LiteralExpression) -> NullableExprKind {
        let (kind, token) = expr.token()?;

        Some(match kind {
            LiteralExpressionKind::DecimalInteger => {
                let text = token.text();

                if text.starts_with('0') && text.len() > 1 {
                    self.diagnostics.push(Diagnostic {
                        message: "Leading '0' can be removed".to_owned(),
                        range: token.text_range(),
                        severity: DiagnosticSeverity::Warning,
                    });
                }
                // Default values are provided to signify that the user has tried
                // to write a literal but the literal was malformed
                // This is to not error out
                let value = text.parse::<i32>().unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(value)))
            }
            LiteralExpressionKind::OctalInteger => {
                let text = token.text();
                // 0321321
                let value = i32::from_str_radix(&text[1..], 8).unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(value)))
            }
            LiteralExpressionKind::HexInteger => {
                let text = token.text();
                //0x12312312
                let value = i32::from_str_radix(&text[2..], 16).unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(value)))
            }
            LiteralExpressionKind::Character => {
                // let text = token.text();
                // let inner = &text[1..];

                // let value = if !inner.starts_with('\\') {
                //     inner.chars().next().map(|c| c as i32)
                // } else {
                //     match inner.chars().nth(1) {
                //         Some('n') => Some('\n' as i32),
                //         Some('t') => Some('\t' as i32),
                //         Some('r') => Some('\r' as i32),
                //         Some('\\') => Some('\\' as i32),
                //         Some('\'') => Some('\'' as i32),

                //         Some('x') => {
                //             let hex = &inner[2..];
                //             u8::from_str_radix(hex, 16).ok().map(|c| c as i32)
                //         }

                //         Some(other) => panic!("unknown escape: {}", other),
                //         None => None,
                //     }
                // }
                // .unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(0)))
            }
            LiteralExpressionKind::Float => {
                let text = token.text();
                let value = text.parse::<f32>().unwrap_or(0.0);

                ExpressionKind::Literal(Type::Float(Some(value)))
            }
            LiteralExpressionKind::String => {
                let string = self.string(&(StringNameKind::Normal, token));

                ExpressionKind::Literal(Type::String {
                    kind: StringKind::Arbitrary,
                    literal: Some(string),
                })
            }
            LiteralExpressionKind::VerbatimString => {
                let string = self.string(&(StringNameKind::Verbatim, token));

                ExpressionKind::Literal(Type::String {
                    kind: StringKind::Arbitrary,
                    literal: Some(string),
                })
            }
            LiteralExpressionKind::Null => ExpressionKind::Literal(Type::Null),
            LiteralExpressionKind::True => ExpressionKind::Literal(Type::Boolean(Some(true))),
            LiteralExpressionKind::False => ExpressionKind::Literal(Type::Boolean(Some(false))),
        })
    }

    fn table_literal_expression(&mut self, expr: &TableLiteralExpression) -> ExpressionKind {
        let table = TableId::new(self.file, self.arena.alloc(TableData::default()));
        let save_symbol = self.container;
        self.container = Container::Table(table);
        for member in expr.members() {
            self.collect_table_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(Type::Table(Some(table)))
    }

    fn class_expression(&mut self, expr: &ClassExpression) -> ExpressionKind {
        let class = self.class(expr);

        let save_symbol = self.container;
        self.container = Container::Class(class);
        for member in expr.members() {
            self.collect_class_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(Type::Class(Some(class)))
    }

    fn array_literal_expression(&mut self, expr: &ArrayLiteralExpression) -> ExpressionKind {
        let mut types: Vec<_> = expr
            .elements()
            .map(|element| self.expr_to_type(&element))
            .collect();

        let Some(mut typ) = types.pop() else {
            return ExpressionKind::Literal(Type::Array(None));
        };

        for next_type in types {
            typ = self.merge_or_union(typ, next_type);
        }

        ExpressionKind::Literal(Type::Array(Some(self.array(ArrayData { typ }))))
    }

    fn name_expression(&mut self, expr: &Name) -> NullableExprKind {
        let ident = expr.identifier()?;
        let text = ident.text();

        let offset = expr.syntax().text_range().end();
        self.resolve_name(text, offset).map(|id| {
            self.new_reference(ident.text_range(), id);
            if matches!(self.get(id).typ, Type::Enum(_))
                && expr
                    .syntax()
                    .parent()
                    .is_some_and(|p| !ast::MemberAccessExpression::can_cast(p.kind()))
            {
                self.diagnostics.push(Diagnostic {
                    message: "'enum' can only appear in property access expression".to_owned(),
                    range: expr.syntax().text_range(),
                    ..Default::default()
                });
            }
            ExpressionKind::Symbol(id)
        })
    }

    fn this_expression(&self, _expr: &ThisExpression) -> ExpressionKind {
        ExpressionKind::Literal(self.execution_container().into())
    }

    fn root_access_expression(&mut self, expr: &RootAccessExpression) -> NullableExprKind {
        let name_token = get_name(expr)?;
        let offset = expr.syntax().text_range().end();

        self.members_of_table(
            self.root_table(),
            FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
            ImportMembers::Root,
        )
        .into_iter()
        .find_map(|(name, id)| {
            if name_token.text() == name.as_ref() {
                self.new_reference(name_token.text_range(), id);
                Some(ExpressionKind::Symbol(id))
            } else {
                None
            }
        })
    }

    fn base_expression(&mut self, expr: &BaseExpression) -> ExpressionKind {
        if let Container::Class(id) = self.execution_container() {
            let class = self.get(id);
            if let Some(inherits) = class.inherits {
                ExpressionKind::Literal(Type::Class(Some(inherits)))
            } else {
                self.diagnostics.push(Diagnostic {
                    message: "Accessing 'base' in a class that doesn't have a superclass"
                        .to_owned(),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
                ExpressionKind::Literal(Type::Null)
            }
        } else {
            self.diagnostics.push(Diagnostic {
                message: "Accessing 'base' inside non-class execution scope".to_owned(),
                range: expr.syntax().text_range(),
                severity: DiagnosticSeverity::Warning,
            });
            ExpressionKind::Literal(Type::Null)
        }
    }

    fn member_access_expression(&mut self, expr: &MemberAccessExpression) -> NullableExprKind {
        let from = self.expr_to_type(&expr.object()?);
        let member_part = expr.member_part()?;
        let name_token = get_name(&member_part)?;

        let offset = expr.syntax().text_range().end();

        let result = self
            .members_of_type(
                from,
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(|(name, id)| {
                if name_token.text() == name.as_ref() {
                    self.new_reference(name_token.text_range(), id);
                    Some(ExpressionKind::Symbol(id))
                } else {
                    None
                }
            });

        if result.is_none() {
            self.no_member_error(from, name_token.text(), expr.syntax().text_range());
        }
        result
    }

    fn element_access_expression(&mut self, expr: &ElementAccessExpression) -> NullableExprKind {
        let from = self.expr_to_type(&expr.object()?);
        let index = expr.index()?.expression()?;

        let Some(ExpressionKind::Literal(Type::String {
            literal: Some(id), ..
        })) = self.collect_expr(&index)
        else {
            return None;
        };

        let string = self.get(id);
        let text = string.text.clone();
        let name_range = string.unquoted_range;
        let offset = expr.syntax().text_range().end();

        let result = self
            .members_of_type(
                from,
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(|(name, id)| {
                if name == text {
                    self.new_reference(name_range, id);
                    Some(ExpressionKind::Symbol(id))
                } else {
                    None
                }
            });

        if result.is_none() {
            self.no_member_error(from, &text, index.syntax().text_range());
        }

        result
    }

    fn special_context(&self, expr: &CallExpression) -> Option<Type> {
        match expr.callee()?.expression()? {
            Expr::MemberAccess(expr) => {
                let object = expr.object()?;
                let expr = self.expr_kind_at(object.syntax().text_range())?;
                Some(self.expr_kind_to_type(Some(expr)))
            }
            Expr::ElementAccess(expr) => {
                let object = expr.object()?;
                let expr = self.expr_kind_at(object.syntax().text_range())?;
                Some(self.expr_kind_to_type(Some(expr)))
            }
            Expr::RootAccess(_) => Some(Type::Table(Some(self.root_table()))),
            _ => None,
        }
    }

    fn call_expression(&mut self, expr: &CallExpression) -> NullableExprKind {
        let obj = self.expr_to_type_with_range(&expr.callee()?.expression()?);

        let arguments: Vec<_> = expr
            .arguments()
            .map(|arg| self.expr_to_type_with_range(&arg))
            .collect();

        let context = self
            .special_context(expr)
            .unwrap_or_else(|| self.execution_container().into());

        Some(ExpressionKind::Literal(
            self.callable(context, obj, &arguments)?,
        ))
    }

    fn clone_expression(&mut self, expr: &CloneExpression) -> NullableExprKind {
        let operand = expr.operand()?;
        let typ = self.expr_to_type(&operand);
        Some(ExpressionKind::Literal(self.clone_type(typ)))
    }

    fn extract_lhs_and_rhs(
        &mut self,
        expr: &BinaryExpression,
    ) -> (Option<TypeWithRange>, Option<TypeWithRange>) {
        let right = expr.rhs().map(|r| self.expr_to_type_with_range(&r));
        let left = expr.lhs().map(|l| self.expr_to_type_with_range(&l));
        (left, right)
    }

    #[allow(clippy::too_many_lines)]
    fn assignment_lhs(&mut self, expr: &Expr) -> Option<AssignmentLeftHandSide> {
        match expr {
            Expr::Name(expr) => {
                let name_token = expr.identifier()?;
                let expr_range = expr.syntax().text_range();
                let offset = expr_range.end();

                let filter = |(name, id): (Box<str>, SymbolId)| {
                    if name_token.text() == name.as_ref() {
                        Some(id)
                    } else {
                        None
                    }
                };

                let locals = self.local_members(offset).into_iter().find_map(filter);

                if let Some(symbol) = locals {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: None,
                        symbol,
                        name_range: expr_range,
                        expr_range,
                    });
                }

                let consts = self
                    .members_of_table(
                        self.const_table(),
                        FindSymbol::OnlyBefore(offset),
                        ImportMembers::Const,
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = consts {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: None,
                        symbol,
                        name_range: expr_range,
                        expr_range,
                    });
                }

                let members = self
                    .members_of_container(
                        self.execution_container(),
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        false,
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = members {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(self.execution_container().into()),
                        symbol,
                        name_range: expr_range,
                        expr_range,
                    });
                }

                let root = self
                    .members_of_table(
                        self.root_table(),
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        ImportMembers::Root,
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = root {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(Type::Table(Some(self.root_table()))),
                        symbol,
                        name_range: expr_range,
                        expr_range,
                    });
                }

                Some(AssignmentLeftHandSide::CanCreate {
                    parent: self.container.into(),
                    new_key: name_token.text().into(),
                    name_range: expr_range,
                    expr_range,
                })
            }
            Expr::MemberAccess(expr) => {
                let obj = self.expr_to_type(&expr.object()?);
                let member_part = expr.member_part()?;

                let name_token = get_name(&member_part)?;
                let expr_range = expr.syntax().text_range();
                let name_range = name_token.text_range();
                let offset = expr_range.end();

                Some(
                    self.members_of_type(
                        obj,
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        false,
                    )
                    .into_iter()
                    .find_map(|(name, id)| {
                        if name_token.text() == name.as_ref() {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: obj,
                            new_key: name_token.text().into(),
                            name_range,
                            expr_range,
                        },
                        |id| AssignmentLeftHandSide::Exists {
                            parent: Some(obj),
                            symbol: id,
                            name_range,
                            expr_range,
                        },
                    ),
                )
            }
            Expr::ElementAccess(expr) => {
                let obj = self.expr_to_type(&expr.object()?);
                let index = expr.index()?.expression()?;
                let expr_range = expr.syntax().text_range();
                let kind = self.collect_expr(&index);
                let Some(ExpressionKind::Literal(Type::String {
                    literal: Some(id), ..
                })) = kind
                else {
                    return Some(AssignmentLeftHandSide::NonStringName {
                        parent: obj,
                        name: TypeWithRange {
                            typ: self.expr_kind_to_type(kind),
                            range: index.syntax().text_range(),
                        },
                        expr_range,
                    });
                };

                let string = self.get(id);
                let text = string.text.clone();
                let name_range = string.unquoted_range;
                let offset = expr_range.end();

                Some(
                    self.members_of_type(
                        obj,
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        false,
                    )
                    .into_iter()
                    .find_map(|(name, id)| if name == text { Some(id) } else { None })
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: obj,
                            new_key: text,
                            name_range,
                            expr_range,
                        },
                        |id| AssignmentLeftHandSide::Exists {
                            parent: Some(obj),
                            symbol: id,
                            name_range,
                            expr_range,
                        },
                    ),
                )
            }
            Expr::RootAccess(expr) => {
                let name_token = get_name(expr)?;
                let expr_range = expr.syntax().text_range();
                let name_range = name_token.text_range();
                let offset = expr_range.end();

                let root = self.root_table();
                Some(
                    self.members_of_table(
                        root,
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        ImportMembers::Root,
                    )
                    .into_iter()
                    .find_map(|(name, id)| {
                        if name_token.text() == name.as_ref() {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: Type::Table(Some(root)),
                            new_key: name_token.text().into(),
                            name_range,
                            expr_range,
                        },
                        |id| AssignmentLeftHandSide::Exists {
                            parent: Some(Type::Table(Some(root))),
                            symbol: id,
                            name_range,
                            expr_range,
                        },
                    ),
                )
            }
            _ => Some(AssignmentLeftHandSide::Invalid(self.collect_expr(expr))),
        }
    }

    fn binary_expression(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            BinaryOperator::NewSlot => self.new_slot_operator(expr),
            BinaryOperator::Assign => self.assign_operator(expr),
            BinaryOperator::Comma => self.comma_operator(expr),
            BinaryOperator::In => Some(self.in_operator(expr)),
            BinaryOperator::InstanceOf => Some(self.instance_of_operator(expr)),
            BinaryOperator::Equals | BinaryOperator::NotEquals => {
                Some(self.equality_operator(expr))
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::ThreeWay => {
                self.comparison_operator(expr);

                Some(ExpressionKind::Literal(
                    if operator == BinaryOperator::ThreeWay {
                        Type::INTEGER
                    } else {
                        Type::BOOLEAN
                    },
                ))
            }
            BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::LeftShift
            | BinaryOperator::RightShift
            | BinaryOperator::UnsignedRightShift => {
                self.bitwise_operator(expr);

                Some(ExpressionKind::Literal(Type::INTEGER))
            }

            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                Some(self.logical_operator(expr))
            }

            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => self.arithmetic_operator(expr, operator),

            BinaryOperator::AddAssign
            | BinaryOperator::SubtractAssign
            | BinaryOperator::MultiplyAssign
            | BinaryOperator::DivideAssign
            | BinaryOperator::ModuloAssign => {
                let right = expr.rhs().map(|r| self.expr_to_type_with_range(&r));
                let left = expr.lhs().and_then(|l| self.assignment_lhs(&l));

                Some(ExpressionKind::Literal(self.arithmetic_assign_operator(
                    left.as_ref(),
                    right.unwrap_or_else(|| TypeWithRange::at_node_end(expr.syntax())),
                    operator,
                )?))
            }
        }
    }

    // Also used by class statement
    fn do_new_slot(
        &mut self,
        whole_expr_range: Option<TextRange>,
        left: Option<AssignmentLeftHandSide>,
        right: TypeWithRange,
        property_kind: PropertyKind,
    ) -> Option<SymbolId> {
        match left {
            Some(AssignmentLeftHandSide::CanCreate {
                parent,
                name_range,
                new_key,
                expr_range,
            }) => {
                let (operand, arguments) =
                    to_operand_and_arguments(parent, expr_range, name_range, right);

                let result = self.new_slot(operand, &arguments);
                if matches!(result, NewSlotResult::NotAllowed) {
                    return None;
                }

                let symbol = self.symbol(Symbol {
                    name: new_key.clone(),
                    typ: right.typ,
                    kind: SymbolKind::Property(property_kind),
                    name_range,
                    range: whole_expr_range.unwrap_or(expr_range),
                    ..Default::default()
                });

                if let Type::Class(Some(id)) = right.typ
                    && let Some(class) = self.get_mut(id)
                    && class.symbol.is_none()
                {
                    class.symbol = Some(symbol);
                }

                if let NewSlotResult::CanAdd(container) = result {
                    self.add_container_member(container, new_key, symbol);

                    if let Type::Function(Some(id)) = right.typ
                        && let Some(function) = self.get_mut(id)
                    {
                        function.container = container;
                    }
                }

                Some(symbol)
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                name_range,
                expr_range,
            }) => {
                if let Some(parent) = parent {
                    // Problematic: when we have something like
                    // ::a <- 1
                    // a <- 1
                    // The `parent` for the second assignment becomes the root which means that the code
                    // below will add the symbol to the root table instead of adding it to the current
                    // `this`, we also can't just map the root to `this` since it doesn't consider
                    // ::a <- 1
                    // ::a <- 1
                    // Where both symbols should go to the root
                    // to solve this we check if name_range == range which distinguishes plain name
                    // expressions from other expressions
                    let parent = if name_range == expr_range {
                        self.execution_container().into()
                    } else {
                        parent
                    };

                    let (operand, arguments) =
                        to_operand_and_arguments(parent, expr_range, name_range, right);

                    let result = self.new_slot(operand, &arguments);
                    if matches!(result, NewSlotResult::NotAllowed) {
                        return None;
                    }

                    let name = self.get(symbol).name.clone();

                    let symbol = self.symbol(Symbol {
                        name: name.clone(),
                        typ: right.typ,
                        kind: SymbolKind::Property(property_kind),
                        name_range,
                        range: whole_expr_range.unwrap_or(expr_range),
                        ..Default::default()
                    });

                    if let Type::Class(Some(id)) = right.typ
                        && let Some(class) = self.get_mut(id)
                        && class.symbol.is_none()
                    {
                        class.symbol = Some(symbol);
                    }

                    if let NewSlotResult::CanAdd(container) = result {
                        self.add_container_member(container, name, symbol);

                        if let Type::Function(Some(id)) = right.typ
                            && let Some(function) = self.get_mut(id)
                        {
                            function.container = container;
                        }
                    }

                    Some(symbol)
                } else {
                    self.new_reference(name_range, symbol);
                    // Parent is only None for locals and consts
                    // ```
                    // local a = 2
                    // a <- 1
                    // ```
                    // is illegal
                    self.diagnostics.push(Diagnostic {
                        message: "Cannot create a new slot with the same name as a local or constant due to the resolution precedence. Prepend variable name with `this.` if you wish to do that".to_owned(),
                        range: name_range,
                        ..Default::default()
                    });
                    None
                }
            }
            Some(AssignmentLeftHandSide::NonStringName {
                parent,
                name: key,
                expr_range,
            }) => {
                let arguments = [key, right];
                let operand = TypeWithRange {
                    typ: parent,
                    range: expr_range,
                };
                self.new_slot(operand, &arguments);

                if let Ok(container) = Container::try_from(parent)
                    && let Type::Function(Some(id)) = right.typ
                    && let Some(function) = self.get_mut(id)
                {
                    function.container = container;
                }
                None
            }
            _ => None,
        }
    }

    fn new_slot_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        let left = expr.lhs().and_then(|l| self.assignment_lhs(&l));

        let right = right_kind.map_or_else(
            || TypeWithRange::at_node_end(expr.syntax()),
            |r| TypeWithRange {
                typ: self.expr_kind_to_type(Some(r)),
                range: expr
                    .rhs()
                    .expect(
                        "For right_kind to be Some, rhs has to exist in order to do collect_expr",
                    )
                    .syntax()
                    .text_range(),
            },
        );

        if let Some(symbol) = self.do_new_slot(
            Some(expr.syntax().text_range()),
            left,
            right,
            PropertyKind::NewSlot,
        ) {
            self.resolve_variable_doc(symbol, expr);
        }

        right_kind
    }

    fn assign_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        let left = expr.lhs().and_then(|l| self.assignment_lhs(&l));

        let right = right_kind.map_or_else(
            || TypeWithRange::at_node_end(expr.syntax()),
            |r| TypeWithRange {
                typ: self.expr_kind_to_type(Some(r)),
                range: expr
                    .rhs()
                    .expect(
                        "For right_kind to be Some, rhs has to exist in order to do collect_expr",
                    )
                    .syntax()
                    .text_range(),
            },
        );

        if let Some(container) = lhs_container(left.as_ref())
            && let Type::Function(Some(id)) = right.typ
            && let Some(function) = self.get_mut(id)
        {
            function.container = container;
        }

        match left {
            Some(AssignmentLeftHandSide::CanCreate {
                parent,
                expr_range,
                name_range,
                ..
            }) => {
                let (operand, arguments) =
                    to_operand_and_arguments(parent, expr_range, name_range, right);

                self.set(operand, &arguments);
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                expr_range,
                name_range,
            }) => {
                self.new_reference(name_range, symbol);
                if !self.get(symbol).is_modifiable() {
                    let name = &self.get(symbol).name;
                    self.diagnostics.push(Diagnostic {
                        message: format!("Symbol '{name}' is not modifiable"),
                        range: expr_range,
                        ..Default::default()
                    });
                    return right_kind;
                }

                if let Some(parent) = parent {
                    let (operand, arguments) =
                        to_operand_and_arguments(parent, expr_range, name_range, right);
                    if self.set(operand, &arguments).is_none() {
                        return right_kind;
                    }
                }

                if symbol.file() != self.file {
                    return right_kind;
                }

                if let Some(new) = self.check_or_update_type(
                    self.get(symbol).typ,
                    self.get(symbol).type_state,
                    NewType::NotExplicit(right),
                    CheckTypeSource::Variable,
                ) && let Some(symbol) = self.get_mut(symbol)
                {
                    symbol.typ = new;
                    symbol.type_state = TypeState::Inferred;
                }
            }
            Some(AssignmentLeftHandSide::NonStringName {
                parent,
                name,
                expr_range,
            }) => {
                let arguments = [name, right];
                let operand = TypeWithRange {
                    typ: parent,
                    range: expr_range,
                };
                self.set(operand, &arguments);
            }

            _ => {}
        }
        right_kind
    }

    fn comma_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let (_left, right) = self.extract_lhs_and_rhs(expr);
        Some(ExpressionKind::Literal(right?.typ))
    }

    fn in_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left, Some(right)) = self.extract_lhs_and_rhs(expr) else {
            return ExpressionKind::Literal(Type::Boolean(None));
        };

        match right.typ {
            Type::Array(_) => {
                if let Some(with) = left
                    && !TypeSet::NUMBER.contains(self.to_type_set(with.typ))
                {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to index into an array using '{}' (only integers are applicable)", self.type_to_str(with.typ)),
                        range: with.range,
                        severity: DiagnosticSeverity::Warning,
                    });
                }
            }
            typ => {
                if !TypeSet::VALID_IN_LHS.contains(self.to_type_set(typ)) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "Indexing into '{}' will always return false",
                            self.type_to_str(typ)
                        ),
                        range: right.range,
                        severity: DiagnosticSeverity::Warning,
                    });
                }
            }
        }

        ExpressionKind::Literal(Type::Boolean(None))
    }

    fn instance_of_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left, right) = self.extract_lhs_and_rhs(expr);
        if let Some(left) = left
            && !TypeSet::VALID_INSTANCE_OF_LHS.contains(self.to_type_set(left.typ))
        {
            self.diagnostics.push(Diagnostic { message: format!("Using '{}' as left-hand side of 'instanceof' operator (only 'instance' is applicable)", self.type_to_str(left.typ)), range: left.range, ..Default::default() });
        }

        if let Some(right) = right
            && !TypeSet::VALID_INSTANCE_OF_RHS.contains(self.to_type_set(right.typ))
        {
            self.diagnostics.push(Diagnostic { message: format!("Using '{}' as right-hand side of 'instanceof' operator (only 'class' is applicable)", self.type_to_str(right.typ)), range: right.range, ..Default::default() });
        }

        ExpressionKind::Literal(Type::Boolean(None))
    }

    fn equality_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (_left_kind, _right_kind) = self.extract_lhs_and_rhs(expr);
        ExpressionKind::Literal(Type::Boolean(None))
    }

    fn is_comparable(&mut self, comparable: TypeWithRange) -> bool {
        let set = self.to_type_set(comparable.typ);
        if TypeSet::ANY.contains(set) {
            return false;
        }

        if TypeSet::CAN_COMPARE.contains(set) {
            return true;
        }

        self.diagnostics.push(Diagnostic {
            message: format!(
                "'{}' does not support comparison",
                self.type_to_str(comparable.typ),
            ),
            range: comparable.range,
            ..Default::default()
        });

        false
    }

    fn comparison_operator(&mut self, expr: &BinaryExpression) {
        let (left, right) = match self.extract_lhs_and_rhs(expr) {
            (Some(left), Some(right)) => {
                let produce_right = self.is_comparable(right);
                if !self.is_comparable(left) {
                    return;
                }
                (left, if produce_right { Some(right) } else { None })
            }
            (None, Some(right)) => {
                self.is_comparable(right);
                return;
            }
            (Some(left), None) => {
                if !self.is_comparable(left) {
                    return;
                }
                (left, None)
            }
            (None, None) => return,
        };

        let left_set = self.to_type_set(left.typ);

        if TypeSet::TABLE_OR_INSTANCE.contains(left_set) {
            let arguments = [right.unwrap_or_else(|| TypeWithRange::at_node_end(expr.syntax()))];

            if let Some(ret) = self.call_metamethod(left, "_cmp", &arguments, MetamethodErrors::No)
            {
                if !TypeSet::NUMBER.contains(self.to_type_set(ret)) {
                    self.diagnostics.push(Diagnostic {
                        message: "'_cmp' must return an integer".to_owned(),
                        range: left.range,
                        ..Default::default()
                    });
                }
            } else {
                self.diagnostics.push(Diagnostic {
                    message: if TypeSet::TABLE.contains(left_set) {
                        "Comparing table with no '_cmp' delegate metamethod defined. The result is undetermenistic".to_owned()
                    } else {
                        "Comparing instance with no '_cmp' class metamethod defined. The result is undetermenistic".to_owned()
                    },
                    range: left.range,
                    severity: DiagnosticSeverity::Warning,
                });
            }
        }

        let Some(right) = right else {
            return;
        };

        let right_set = self.to_type_set(right.typ);
        if TypeSet::NULL.contains(left_set) || TypeSet::NULL.contains(right_set) {
            return;
        }

        if TypeSet::are_both_numbers(left_set, right_set) {
            return;
        }

        let intersect = left_set.intersect(right_set);
        if TypeSet::CAN_COMPARE.contains(intersect) {
            return;
        }

        self.diagnostics.push(Diagnostic {
            message: format!(
                "'{}' does not support comparison with '{}'",
                self.type_to_str(left.typ),
                self.type_to_str(right.typ)
            ),
            range: right.range,
            ..Default::default()
        });
    }

    fn has_bitwise_operations(&mut self, operand: TypeWithRange) -> bool {
        let set = self.to_type_set(operand.typ);
        if TypeSet::ANY.contains(set) {
            return false;
        }

        if TypeSet::INTEGER.contains(set) {
            return true;
        }

        self.diagnostics.push(Diagnostic {
            message: format!(
                "'{}' does not support bitwise operations",
                self.type_to_str(operand.typ),
            ),
            range: operand.range,
            ..Default::default()
        });

        false
    }

    fn bitwise_operator(&mut self, expr: &BinaryExpression) {
        let (left, right) = self.extract_lhs_and_rhs(expr);
        if let Some(left) = left {
            self.has_bitwise_operations(left);
        }

        if let Some(right) = right {
            self.has_bitwise_operations(right);
        }
    }

    fn logical_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left, right) = self.extract_lhs_and_rhs(expr);

        ExpressionKind::Literal(self.merge_or_union(
            left.map_or(Type::Unknown, |l| l.typ),
            right.map_or(Type::Unknown, |r| r.typ),
        ))
    }

    fn arithmetic_operator(
        &mut self,
        expr: &BinaryExpression,
        operator: BinaryOperator,
    ) -> NullableExprKind {
        let (left, right) = self.extract_lhs_and_rhs(expr);
        let result = self.arithmetic(left?, right?, operator)?;
        Some(ExpressionKind::Literal(result))
    }

    // This signature is so weird because it is also used by increment / decrement operators
    fn arithmetic_assign_operator(
        &mut self,
        left: Option<&AssignmentLeftHandSide>,
        right: TypeWithRange,
        operator: BinaryOperator,
    ) -> Option<Type> {
        match left {
            Some(AssignmentLeftHandSide::CanCreate {
                parent,
                expr_range,
                name_range,
                ..
            }) => {
                let (operand, arguments) =
                    to_operand_and_arguments(*parent, *expr_range, *name_range, right);
                self.set(operand, &arguments);
                None
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                expr_range,
                name_range,
            }) => {
                self.new_reference(*name_range, *symbol);
                let typ = self.arithmetic(
                    TypeWithRange {
                        typ: self.get(*symbol).typ,
                        range: *name_range,
                    },
                    right,
                    operator,
                )?;

                let type_with_range = TypeWithRange {
                    typ,
                    range: *expr_range,
                };

                if !self.get(*symbol).is_modifiable() {
                    let name = &self.get(*symbol).name;
                    self.diagnostics.push(Diagnostic {
                        message: format!("Symbol '{name}' is not modifiable"),
                        range: *name_range,
                        ..Default::default()
                    });
                    return Some(typ);
                }

                if let Some(parent) = parent {
                    let (operand, arguments) = to_operand_and_arguments(
                        *parent,
                        *expr_range,
                        *name_range,
                        type_with_range,
                    );
                    if self.set(operand, &arguments).is_none() {
                        return Some(typ);
                    }
                }

                if symbol.file() != self.file {
                    return Some(typ);
                }

                if let Some(new) = self.check_or_update_type(
                    self.get(*symbol).typ,
                    self.get(*symbol).type_state,
                    NewType::NotExplicit(type_with_range),
                    CheckTypeSource::Variable,
                ) && let Some(symbol) = self.get_mut(*symbol)
                {
                    symbol.typ = new;
                    symbol.type_state = TypeState::Inferred;
                }
                Some(typ)
            }
            Some(AssignmentLeftHandSide::NonStringName {
                parent,
                name,
                expr_range,
            }) => {
                let operand = TypeWithRange {
                    typ: *parent,
                    range: *expr_range,
                };
                let typ = self.arithmetic(operand, right, operator)?;

                let type_with_range = TypeWithRange {
                    typ,
                    range: *expr_range,
                };

                let (operand, arguments) =
                    to_operand_and_arguments(*parent, *expr_range, name.range, type_with_range);
                self.set(operand, &arguments);
                Some(typ)
            }

            _ => None,
        }
    }

    fn conditional_expression(&mut self, expr: &ConditionalExpression) -> ExpressionKind {
        if let Some(expr) = expr.condition() {
            self.collect_expr(&expr);
        }

        let then_type = expr
            .then_branch()
            .and_then(|b| b.expression())
            .map_or(Type::Unknown, |expr| self.expr_to_type(&expr));

        let else_type = expr
            .else_branch()
            .and_then(|b| b.expression())
            .map_or(Type::Unknown, |expr| self.expr_to_type(&expr));

        ExpressionKind::Literal(self.merge_or_union(then_type, else_type))
    }

    fn prefix_unary_expression(&mut self, expr: &PrefixUnaryExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            PrefixUnaryOperator::Negation => self.negation_operator(expr),
            PrefixUnaryOperator::BitwiseNot => {
                self.bitwise_not_operator(expr);
                Some(ExpressionKind::Literal(Type::Integer(None)))
            }
            PrefixUnaryOperator::LogicalNot => {
                self.logical_not_operator(expr);

                Some(ExpressionKind::Literal(Type::Boolean(None)))
            }
        }
    }

    fn negation_operator(&mut self, expr: &PrefixUnaryExpression) -> NullableExprKind {
        let operand = self.expr_to_type_with_range(&expr.operand()?);

        Some(ExpressionKind::Literal(match operand.typ {
            Type::Integer(Some(value)) => Type::Integer(Some(-value)),
            Type::Float(Some(value)) => Type::Float(Some(-value)),
            _ => {
                let set = self.to_type_set(operand.typ);
                if TypeSet::NUMBER.contains(set) {
                    operand.typ
                } else {
                    self.call_metamethod(
                        operand,
                        "_unm",
                        &Vec::new(),
                        MetamethodErrors::Yes {
                            keyword: "negation",
                        },
                    )?
                }
            }
        }))
    }

    fn bitwise_not_operator(&mut self, expr: &PrefixUnaryExpression) {
        let Some(operand) = expr.operand() else {
            return;
        };
        let operand = self.expr_to_type_with_range(&operand);
        self.has_bitwise_operations(operand);
    }

    fn logical_not_operator(&mut self, expr: &PrefixUnaryExpression) {
        let Some(operand) = expr.operand() else {
            return;
        };
        self.collect_expr(&operand);
    }

    fn prefix_update_expression(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            PrefixUpdateOperator::Increment => self.prefix_increment_operator(expr),
            PrefixUpdateOperator::Decrement => self.prefix_decrement_operator(expr),
        }
    }

    fn prefix_increment_operator(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        Some(ExpressionKind::Literal(self.arithmetic_assign_operator(
            operand.as_ref(),
            TypeWithRange {
                typ: Type::Integer(Some(1)),
                range: expr.syntax().text_range(),
            },
            BinaryOperator::AddAssign,
        )?))
    }

    fn prefix_decrement_operator(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);

        Some(ExpressionKind::Literal(self.arithmetic_assign_operator(
            operand.as_ref(),
            TypeWithRange {
                typ: Type::Integer(Some(1)),
                range: expr.syntax().text_range(),
            },
            BinaryOperator::SubtractAssign,
        )?))
    }

    fn postfix_update_expression(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            PostfixUpdateOperator::Increment => self.postfix_increment_operator(expr),
            PostfixUpdateOperator::Decrement => self.postfix_decrement_operator(expr),
        }
    }

    fn postfix_increment_operator(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let kind = operand.as_ref().and_then(NullableExprKind::from);
        self.arithmetic_assign_operator(
            operand.as_ref(),
            TypeWithRange {
                typ: Type::Integer(Some(1)),
                range: expr.syntax().text_range(),
            },
            BinaryOperator::AddAssign,
        );
        kind
    }

    fn postfix_decrement_operator(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let kind = operand.as_ref().and_then(NullableExprKind::from);
        self.arithmetic_assign_operator(
            operand.as_ref(),
            TypeWithRange {
                typ: Type::Integer(Some(1)),
                range: expr.syntax().text_range(),
            },
            BinaryOperator::SubtractAssign,
        );
        kind
    }

    fn delete_expression(&mut self, expr: &DeleteExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let kind = operand.as_ref().and_then(NullableExprKind::from);
        match operand {
            Some(AssignmentLeftHandSide::CanCreate {
                parent,
                expr_range,
                name_range,
                ..
            }) => {
                let delete_operand = TypeWithRange {
                    typ: parent,
                    range: expr_range,
                };
                let index = TypeWithRange {
                    typ: Type::STRING,
                    range: name_range,
                };
                self.delete(delete_operand, index);

                kind
            }
            Some(AssignmentLeftHandSide::NonStringName {
                parent,
                expr_range,
                name: key,
                ..
            }) => {
                let delete_operand = TypeWithRange {
                    typ: parent,
                    range: expr_range,
                };
                self.delete(delete_operand, key);

                kind
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                expr_range,
                name_range,
            }) => {
                self.new_reference(name_range, symbol);
                if let Some(parent) = parent {
                    let delete_operand = TypeWithRange {
                        typ: parent,
                        range: expr_range,
                    };
                    let index = TypeWithRange {
                        typ: Type::STRING,
                        range: name_range,
                    };
                    self.delete(delete_operand, index);

                    return Some(ExpressionKind::Literal(self.get(symbol).typ));
                }
                // ```
                // local a = 2
                // delete a
                // ```
                // is illegal
                self.diagnostics.push(Diagnostic {
                    message: "Cannot delete a variable with the same name as a local or constant due to the resolution precedence. Prepend variable name with `this.` if you wish to do that".to_owned(),
                    range: name_range,
                    ..Default::default()
                });

                Some(ExpressionKind::Literal(self.get(symbol).typ))
            }
            _ => None,
        }
    }

    fn type_of_expression(&mut self, expr: &TypeOfExpression) -> ExpressionKind {
        let Some(operand) = expr.operand().map(|o| self.expr_to_type_with_range(&o)) else {
            return ExpressionKind::Literal(Type::STRING);
        };

        ExpressionKind::Literal(
            self.call_metamethod(operand, "_typeof", &Vec::new(), MetamethodErrors::No)
                .unwrap_or(Type::STRING),
        )
    }

    fn resume_expression(&mut self, expr: &ResumeExpression) -> NullableExprKind {
        let typ = self.expr_to_type(&expr.operand()?);

        match typ {
            Type::Unknown | Type::Any => None,
            Type::Generator(id) => Some(ExpressionKind::Literal(self.get(id?).yields)),
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Only generators can be resumed".to_owned(),
                    range: expr.syntax().text_range(),
                    ..Default::default()
                });
                None
            }
        }
    }

    fn raw_call_expression(&mut self, expr: &RawCallExpression) -> NullableExprKind {
        let mut arguments: Vec<_> = expr
            .arguments()
            .map(|arg| self.expr_to_type_with_range(&arg))
            .collect();

        if arguments.len() < 2 {
            self.diagnostics.push(Diagnostic {
                message: "'rawcall' requires at least 2 parameters: function to call and context"
                    .to_owned(),
                range: expr.syntax().text_range(),
                ..Default::default()
            });
            return None;
        }

        let function = arguments.remove(0);
        let context = arguments.remove(0);

        let obj = function;
        Some(ExpressionKind::Literal(self.callable(
            context.typ,
            obj,
            &arguments,
        )?))
    }

    fn parenthesised_expression(&mut self, expr: &ParenthesisedExpression) -> NullableExprKind {
        let expr = expr.inner()?;
        self.collect_expr(&expr)
    }

    fn function_expression(&mut self, expr: &FunctionExpression) -> ExpressionKind {
        let id = self.collect_function(expr);
        ExpressionKind::Literal(Type::Function(Some(id)))
    }

    fn lambda_expression(&mut self, expr: &LambdaExpression) -> ExpressionKind {
        let id = self.collect_function(expr);
        ExpressionKind::Literal(Type::Function(Some(id)))
    }

    fn include_script(&mut self, arguments: &[TypeWithRange]) {
        let Some(path_string) = arguments.first() else {
            // Case with no path will be handled in call_type
            return;
        };

        let Type::String { literal: str, .. } = path_string.typ else {
            // Same as above
            return;
        };

        let Some(id) = str else {
            self.diagnostics.push(Diagnostic {
                message: "Could not resolve the path statically, symbols will not be included"
                    .to_owned(),
                range: path_string.range,
                severity: DiagnosticSeverity::Information,
            });
            return;
        };

        let path = PathBuf::from(self.get(id).text.to_string());

        let Ok(file) = self.db.get_script(path) else {
            return;
        };

        let target = match arguments.get(1) {
            Some(expr) => {
                // if expr.typ == Type::Unknown {
                //     match self.execution_container() {
                //         Container::Table(id) => ImportTarget::Table(id),
                //         Container::Class(id) => ImportTarget::Class(id),
                //         Container::Instance(id) => ImportTarget::Class(id),
                //         Container::Enum(_) => {
                //             self.diagnostics.push(Diagnostic {
                //                 message: format!("Type 'enum' cannot receive new members"),
                //                 range: expr.range,
                //                 severity: DiagnosticSeverity::Warning,
                //             });
                //             return;
                //         }
                //     }
                // } else {
                let Ok(target) = ImportTarget::try_from(expr.typ) else {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "Type '{}' cannot receive new members",
                            self.type_to_str(expr.typ)
                        ),
                        range: expr.range,
                        severity: DiagnosticSeverity::Warning,
                    });
                    return;
                };
                target
                // }
            }

            None => match self.execution_container() {
                Container::Table(id) => ImportTarget::Table(id),
                Container::Class(id) | Container::Instance(id) => ImportTarget::Class(id),
                Container::Enum(_) => return,
            },
        };

        self.imports
            .entry(target)
            .and_modify(|e| e.push(file))
            .or_insert_with(|| vec![file]);
    }

    fn set_delegate(&mut self, context: Type, arguments: &[TypeWithRange]) {
        let Some(first) = arguments.first() else {
            return;
        };

        let Type::Table(delegate) = first.typ else {
            return;
        };

        let Type::Table(Some(for_table)) = context else {
            return;
        };

        if let Some(table) = self.get_mut(for_table) {
            table.delegate = delegate;
        }
    }

    fn bindenv(&mut self, context: Type, arguments: &[TypeWithRange]) -> Type {
        let Some(first) = arguments.first() else {
            return context;
        };

        let Ok(container) = Container::try_from(first.typ) else {
            return context;
        };

        let Type::Function(Some(function_id)) = context else {
            return context;
        };

        let old = self.get(function_id);
        let new = FunctionId::new(
            self.file,
            self.arena.alloc(FunctionData {
                container,
                ..old.clone()
            }),
        );

        Type::Function(Some(new))
    }

    fn unused_variables_diagnostics(&mut self) {
        for (id, references) in &self.symbol_to_ranges {
            if references.len() > 1 {
                continue;
            }

            let symbol = self.get(*id);
            if symbol.name.starts_with('_') {
                continue;
            }

            self.diagnostics.push(Diagnostic {
                message: match symbol.kind {
                    SymbolKind::Local(LocalKind::Function | LocalKind::Variable) => {
                        format!("Unused local variable '{}'", symbol.name)
                    }
                    SymbolKind::Local(LocalKind::Parameter) => {
                        format!(
                            "Unused parameter '{}'. Prepend the name with '_' if it cannot be removed",
                            symbol.name
                        )
                    }
                    // SymbolKind::Local(LocalKind::VariedArgs) => {
                    //     "Unused variable arguments".to_owned()
                    // }
                    _ => continue
                },
                range: symbol.name_range,
                severity: DiagnosticSeverity::Unnecessary,
            });
        }
    }

    fn deprecated_diagnostics(&mut self) {
        for (id, references) in &self.symbol_to_ranges {
            let symbol = self.get(*id);
            if !symbol.flags.contains(SymbolFlags::DEPRECATED) {
                continue;
            }

            let message = format!("'{}' is deprecated", symbol.name);

            let mut references = references.iter().copied();
            // Skip the definition
            if id.file() == self.file() {
                references.next();
            }

            for reference in references {
                self.diagnostics.push(Diagnostic {
                    message: message.clone(),
                    range: reference,
                    severity: DiagnosticSeverity::Deprecated,
                });
            }
        }
    }
}

fn parent_doc(node: &SyntaxNode) -> Option<DocComment> {
    let parent = node.parent()?;
    // /** ... */
    // new <- function() {}
    if let Some(bin) = BinaryExpression::cast(parent.clone()) {
        return bin.doc();
    }

    // class a = {
    //    /** ... */
    //    prop = function() {}
    // }
    if let Some(prop) = Property::cast(parent.clone()) {
        return prop.doc();
    }

    // Initially wrapped in 'Initialiser' node
    let parent = parent.parent()?;
    let init = VariableDeclaration::cast(parent.clone())?;

    // local
    // /** ... */
    // a = function() {}
    init.doc().or_else(||
                    // /** ... */
                    // local a = function() {}
                    LocalVariableDeclaration::cast(parent.parent()?)?.doc())
}
