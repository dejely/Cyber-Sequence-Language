if exists('b:current_syntax')
  finish
endif

syntax case match
syntax iskeyword A-Z,a-z,48-57,_

syntax region cslString start=+"+ end=+"+
syntax sync fromstart

syntax keyword cslDeclaration var
syntax keyword cslDeclaration fun nextgroup=cslFunctionName skipwhite skipnl skipempty
syntax match cslFunctionName '\<[A-Za-z_][A-Za-z0-9_]*\>' contained
syntax keyword cslControl if else for while return print
syntax keyword cslReserved class super this watch ability target source sequence require
syntax keyword cslReserved capability count by within alert inspect isolate execute

syntax keyword cslBoolean true false
syntax keyword cslNil nil
syntax match cslNumber '\<[0-9]\+\%(\.[0-9]\+\)\?'
syntax match cslAnnotation '@[ \t]*\%(doc\|deprecated\)\>'
syntax match cslCall '\<[A-Za-z_][A-Za-z0-9_]*\ze[ \t]*('
syntax keyword cslBuiltin len typeOf
syntax keyword cslOperator and or
syntax match cslOperator '[=!<>]=\?\|[-+*/]'
syntax match cslComment '//.*$'

highlight default link cslComment Comment
highlight default link cslString String
highlight default link cslDeclaration Keyword
highlight default link cslFunctionName Function
highlight default link cslControl Statement
highlight default link cslReserved Keyword
highlight default link cslBoolean Boolean
highlight default link cslNil Constant
highlight default link cslNumber Number
highlight default link cslAnnotation PreProc
highlight default link cslCall Function
highlight default link cslBuiltin Function
highlight default link cslOperator Operator

let b:current_syntax = 'csl'
