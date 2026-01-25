use std::borrow::Cow;
use std::path::Path;
// use std::path::{Path, PathBuf};

use concat_string::concat_string;
use itertools::Itertools;
use thiserror::Error;

#[cfg(feature = "vfs")]
use vfs::VfsPath;

#[cfg(not(feature = "vfs"))]
type FeatPath = std::path::Path;
#[cfg(feature = "vfs")]
type FeatPath = vfs::VfsPath;

fn read_to_string(input_file: &FeatPath) -> Result<String> {
    #[cfg(not(feature = "vfs"))] {
        return read_to_string_std(input_file);
    }
    #[cfg(feature = "vfs")] {
        let mut result = String::new();
        input_file.open_file()?.read_to_string(&mut result)
            .map_err(|err| Error::IOError(err, input_file.as_str().into()))?;
        return Ok(result);
    }
}

fn read_to_string_std(input_file: &Path) -> Result<String> {
    return std::fs::read_to_string(input_file)
        .map_err(|err| Error::IOError(err, input_file.to_path_buf()));
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid macro `{}` on line {}", .0, .1)]
    InvalidMacro(String, usize),
    #[error("Found extra parameters in #param macro on line {}", .0)]
    ExtraParamsInParamMacro(usize),
    #[error("Not enough parameters passed to template file")]
    NotEnoughParameters,
    #[error("Unused parameters passed to template file")]
    UnusedParameters,
    #[error("Not enough parameters passed to function-like macro `{}`", .0)]
    NotEnoughParametersMacro(String),
    #[error("Too many parameters passed to function-like macro `{}`", .0)]
    UnusedParametersMacro(String),
    #[error("Invalid parameter name {} on line {}", .0, .1)]
    InvalidParameterName(String, usize),
    #[error("First parameter of #include should be a string on line {}", .0)]
    FirstParamOfIncludeNotString(usize),
    #[error("IOError while reading {}: {}", .1.display(), .0)]
    IOError(std::io::Error, std::path::PathBuf),
    #[cfg(feature = "vfs")]
    #[error("VfsError: {}", .0)]
    VfsError(#[from] vfs::VfsError),
}

type Result<T> = std::result::Result<T, Error>;

/// Parses a file using the templating engine.
///
/// For an example, see [parse_string].
///
/// # Parameters
/// - `input_file`: the file that is read
/// - `base_dir`: all includes are resolved relative to this directory
/// - `parameters`: if `input_file` contains any parameter macros, pass an iterator
///                 to them here. Otherwise pass `std::iter::empty()`.
pub fn parse<'a>(
    input_file: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
    parameters: impl Iterator<Item = &'a str>,
) -> Result<String> {
    return parse_cow(input_file, base_dir, parameters);
}

/// Parse a file in a virtual filesystem using the templating engine.
///
/// # Examples
///
/// ```rust
/// # use ppx_impl::parse_vfs;
/// # use vfs::VfsPath;
/// let root: VfsPath = vfs::MemoryFS::new().into();
/// let path = root.join("test.txt").unwrap();
///
/// path.create_file().unwrap().write_all(b"#param A\nHello A!");
/// let result = parse_vfs(path, root, ["world"].into_iter()).unwrap();
/// assert_eq!(result, "Hello world!");
/// ```
#[cfg(feature = "vfs")]
pub fn parse_vfs<'a>(
    input_file: impl Into<VfsPath>,
    base_dir: impl Into<VfsPath>,
    parameters: impl Iterator<Item = &'a str>
) -> Result<String> {
    return parse_vfs_cow(input_file.into(), base_dir.into(), parameters);
}

/// Same as [parse], but the parameters iterator has an item of `impl Into<Cow<str>>`.
/// This is so that an empty iterator can be passed to `parse`.
pub fn parse_cow<'a, Iter, C>(
    input_file: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
    parameters: Iter
) -> Result<String>
    where
        Iter: Iterator<Item = C>,
        C: Into<Cow<'a, str>>
{
    let content = read_to_string_std(input_file.as_ref())?;

    return parse_string_cow(&content, base_dir, parameters);
}

#[cfg(feature = "vfs")]
pub fn parse_vfs_cow<'a, Iter, C>(
    input_file: impl Into<VfsPath>,
    base_dir: impl Into<VfsPath>,
    parameters: Iter
) -> Result<String>
    where
        Iter: Iterator<Item = C>,
        C: Into<Cow<'a, str>>
{
    let content = read_to_string(&input_file.into())?;

    return parse_string_vfs_cow(&content, base_dir, parameters);
}

/// Parses a file using the templating engine.
///
/// # Parameters
/// - `input`: the contents to process
/// - `base_dir`: all includes are resolved relative to this directory
/// - `parameters`: if `input` contains any parameter macros, pass an iterator
///                 to them here. Otherwise pass `std::iter::empty()`.
///
/// # Example
///
/// ```rust
/// # use ppx_impl::*;
/// # #[cfg(not(feature = "vfs"))]
/// let res = parse_string(
///     "#define A 4\nThe answer is A",
///     std::env::current_dir().unwrap(),
///     std::iter::empty()
/// ).unwrap();
/// # #[cfg(not(feature = "vfs"))]
/// assert_eq!(res, "The answer is 4");
/// ```
pub fn parse_string<'a>(
    input: &str,
    base_dir: impl AsRef<Path>,
    parameters: impl Iterator<Item = &'a str>
) -> Result<String> {
    parse_string_cow(input, base_dir, parameters)
}

#[cfg(feature = "vfs")]
pub fn parse_string_vfs<'a>(
    input: &str,
    base_dir: impl Into<VfsPath>,
    parameters: impl Iterator<Item = &'a str>
) -> Result<String> {
    parse_string_vfs_cow(input, base_dir, parameters)
}

/// Same as [parse_string], but the parameters iterator has an item of `impl Into<Cow<str>>`.
/// This is so that an empty iterator can be passed to `parse_string`.
pub fn parse_string_cow<'a, Iter, C>(
    input: &str,
    base_dir: impl AsRef<Path>,
    parameters: Iter
) -> Result<String>
    where
        Iter: Iterator<Item = C>,
        C: Into<Cow<'a, str>>
{
    #[cfg(not(feature = "vfs"))]
    let base_dir = base_dir.as_ref();
    #[cfg(feature = "vfs")]
    let base_dir = VfsPath::from(vfs::PhysicalFS::new(base_dir));
    parse_string_cow_impl(input, &base_dir, &mut parameters.map(|v| v.into()))
}

#[cfg(feature = "vfs")]
pub fn parse_string_vfs_cow<'a, Iter, C>(
    input: &str,
    base_dir: impl Into<VfsPath>,
    parameters: Iter
) -> Result<String>
    where
        Iter: Iterator<Item = C>,
        C: Into<Cow<'a, str>>
{
    parse_string_cow_impl(input, &base_dir.into(), &mut parameters.map(|v| v.into()))
}

fn parse_string_cow_impl<'a>(
    input: &str,
    base_dir: &FeatPath,
    parameters: &mut dyn Iterator<Item = Cow<'a, str>>
) -> Result<String> {
    let mut out = String::new();

    let mut replacements: Vec<(String, Cow<str>)> = vec![];
    let mut fn_replacements: Vec<(String, Vec<String>, String)> = vec![];
    let mut cur_fn_replacement: Option<(String, Vec<String>, String)> = None;

    let max_lines = input.chars()
        .filter(|c| *c == '\n')
        .count();

    for (line_num, line) in input.lines().enumerate() {
        if let Some(cur_fn_repl) = cur_fn_replacement {
            if line.ends_with("\\") {
                cur_fn_replacement = Some((cur_fn_repl.0, cur_fn_repl.1, cur_fn_repl.2 + &line[..line.len()-1]))
            } else {
                fn_replacements.push((cur_fn_repl.0, cur_fn_repl.1, cur_fn_repl.2 + line));
                cur_fn_replacement = None;
            }
            continue;
        }

        let mut line_chars = line.chars().skip_while(char::is_ascii_whitespace);
        let start_char = line_chars.by_ref().next();
        match start_char {
            Some('#') => {
                let macro_name = line_chars.by_ref().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>();
                match macro_name.as_str() {
                    "define" => {
                        let mut is_last_bracket = false;
                        let name = line_chars.by_ref()
                            .skip_while(char::is_ascii_whitespace)
                            .take_while(|c| {
                                if *c == '(' {
                                    is_last_bracket = true;
                                }
                                !c.is_ascii_whitespace() && *c != '('
                            })
                            .collect::<String>();

                        if is_last_bracket {
                            let params = line_chars.by_ref()
                                .take_while(|c| *c != ')')
                                .chunk_by(|c| *c == ',');
                            let params = params
                                .into_iter()
                                .filter(|(b, _)| !b)
                                .map(|(_, i)| i
                                    .skip_while(char::is_ascii_whitespace)
                                    .take_while(|c| !c.is_ascii_whitespace())
                                    .collect::<String>())
                                .collect::<Vec<String>>();

                            let check_param_name = params.iter().find(|param| !param.chars().all(|c| c.is_alphanumeric() || c == '_'))
                                .or(params.iter().find(|param| param.len() == 0 || param.chars().next().unwrap().is_numeric()));
                            if let Some(param_name) = check_param_name {
                                return Err(Error::InvalidParameterName(param_name.clone(), line_num))
                            }

                            let replacement = line_chars.by_ref().collect::<String>();

                            if replacement.ends_with("\\") {
                                cur_fn_replacement = Some((name, params, replacement[..replacement.len()-1].to_string()));
                            } else {
                                fn_replacements.push((name, params, replacement));
                            }
                        } else {
                            let replacement = line_chars.collect::<Cow<str>>();
                            replacements.push((name, replacement))
                        }
                    }, "include" => {
                        let path = line_chars.by_ref()
                            .skip_while(char::is_ascii_whitespace)
                            .take_while(|c| !c.is_ascii_whitespace())
                            .collect::<String>();

                        if !(path.starts_with('"') && path.ends_with('"')) {
                            return Err(Error::FirstParamOfIncludeNotString(line_num));
                        }

                        let path = &path[1..path.len()-1];

                        let params = line_chars.by_ref()
                            .chunk_by(|c| *c == ',');
                        let mut params = params
                            .into_iter()
                            .filter(|(b, _)| !b)
                            .map(|(_, i)| Cow::Owned(i.collect::<String>()));

                        #[cfg(not(feature = "vfs"))]
                        let file_path = base_dir.join(path);

                        #[cfg(feature = "vfs")]
                        let file_path = base_dir.join(path)?;

                        let content = read_to_string(&file_path)?;

                        out += parse_string_cow_impl(&content, &base_dir, &mut params)?.as_str();
                    }, "param" => {
                        let param_name = line_chars.by_ref()
                            .skip_while(|c| c.is_ascii_whitespace())
                            .take_while(|c| !c.is_ascii_whitespace())
                            .collect::<String>();

                        if !line_chars.by_ref().all(|c| c.is_ascii_whitespace()) {
                            return Err(Error::ExtraParamsInParamMacro(line_num));
                        }

                        let Some(param_value) = parameters.next() else {
                            return Err(Error::NotEnoughParameters);
                        };

                        replacements.push((param_name, param_value.into()));
                    },
                    _ => return Err(Error::InvalidMacro(macro_name, line_num)),
                }
            },
            Some('\\') if (line_chars.next() == Some('#')) => {
                out += fn_replace(replace(&line.replacen("\\#", "#", 1), &replacements), &fn_replacements)?.as_ref();
                if line_num != max_lines {
                    out += "\n";
                }
            },
            _ => {
                out += fn_replace(replace(line, &replacements), &fn_replacements)?.as_ref();
                if line_num != max_lines {
                    out += "\n";
                }
            },
        }
    }

    if parameters.count() != 0 {
        return Err(Error::UnusedParameters);
    }

    return Ok(out);
}

fn is_ident(str: &str, start: usize, end: usize) -> bool {
    (start == 0 || str.chars().nth(start - 1).map(|c| !(c.is_alphanumeric() || c == '_')).unwrap_or(true))
        && str.chars().nth(end).map(|c| !(c.is_alphanumeric() || c == '_')).unwrap_or(true)
}

fn replace<'a>(line: &'a str, replacements: &Vec<(String, Cow<str>)>) -> Cow<'a, str> {
    let mut out: Cow<str> = line.into();

    for replacement in replacements {
        out = replace_all(out, &replacement.0, replacement.1.as_ref(), is_ident);
    }

    return out;
}

fn fn_replace<'a>(line: Cow<'a, str>, replacements: &Vec<(String, Vec<String>, String)>) -> Result<Cow<'a, str>> {
    let mut out: Cow<str> = line;

    for replacement in replacements {
        out = replace_all_fn(out, replacement.0.as_str(), replacement.2.as_str(), &replacement.1, is_ident)?;
    }

    return Ok(out);
}

fn replace_all<'a>(str: Cow<'a, str>, to_match: &str, replacement: &str, predicate: impl Fn(&str, usize, usize) -> bool) -> Cow<'a, str> {
    let matches = str.match_indices(to_match).collect::<Vec<_>>();

    let mut out: Option<Cow<str>> = None;
    let mut end_idx = str.len();

    for (idx, _) in matches.into_iter().rev() {
        if predicate(str.as_ref(), idx, idx + to_match.len()) {
            let following_str = &str[idx + to_match.len()..end_idx];
            end_idx = idx;
            out = out.map(|m| concat_string!(replacement, following_str, m.as_ref()).into())
                .or(Some(concat_string!(replacement, following_str).into()));
        }
    }

    if end_idx != 0 {
        out = out.map(|m| concat_string!(&str[0..end_idx], m.as_ref()).into())
    }

    return out.unwrap_or(str);
}

fn replace_all_fn<'a>(
    str: Cow<'a, str>,
    name: &str,
    replacement: &str,
    param_names: &Vec<String>,
    predicate: impl Fn(&str, usize, usize) -> bool,
) -> Result<Cow<'a, str>> {
    let matches = str.match_indices(name).collect::<Vec<_>>();

    let mut out: Option<Cow<str>> = None;
    let mut end_idx = str.len();

    for (idx, _) in matches.into_iter().rev() {
        let mut iter = str.chars();
        if iter.by_ref().nth(idx + name.len()) != Some('(') {
            continue;
        }

        let mut parens = 0;
        let mut params = Vec::new();
        let mut cur = String::new();
        let mut param_len = 0;
        for (i, c) in iter.enumerate() {
            if c == ')' {
                parens -= 1;
                if parens == -1 {
                    params.push(cur);
                    cur = String::new();
                    param_len = i;
                    break;
                }
            }

            // next param
            if c == ',' && parens == 0 {
                params.push(cur);
                cur = String::new();
                continue;
            }

            if c == '(' {
                parens += 1;
            }

            cur.push(c);
        }

        let to_replace_len = name.len() + 2 + param_len;

        if !predicate(str.as_ref(), idx, idx + to_replace_len) {
            continue;
        }

        if params.len() < param_names.len() {
            return Err(Error::NotEnoughParametersMacro(name.to_string()));
        } else if params.len() > param_names.len() {
            return Err(Error::UnusedParametersMacro(name.to_string()))
        }

        let params = param_names.iter()
            .zip(params.iter());

        let mut replacement = Cow::Borrowed(replacement);
        for param in params {
            replacement = replace_all(replacement, param.0, param.1, is_ident);
        }

        let following_str = &str[idx + to_replace_len..end_idx];
        end_idx = idx;
        out = out.map(|m| concat_string!(replacement, following_str, m.as_ref()).into())
            .or(Some(concat_string!(replacement, following_str).into()));
    }

    if end_idx != 0 {
        out = out.map(|m| concat_string!(&str[0..end_idx], m.as_ref()).into());
    }

    return Ok(out.unwrap_or(str));
}
