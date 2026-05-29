import parser from 'vue-eslint-parser';
import { FlatCompat } from '@eslint/eslintrc';

const compat = new FlatCompat();

export default [
  {
    ignores: [
      '**/*.sh',
      '**/node_modules',
      '**/*.md',
      '**/*.woff',
      '**/*.ttf',
      '**/.vscode',
      '**/.idea',
      '**/dist',
      'public',
      'docs',
      '**/.husky',
      '**/.local',
      'bin',
      '**/Dockerfile',
      '**/components.d.ts',
    ],
  },
  ...compat.extends(
    'plugin:vue/recommended',
    'plugin:@typescript-eslint/recommended',
    'prettier'
  ),
  {
    languageOptions: {
      parser: parser,
      ecmaVersion: 2020,
      sourceType: 'module',
      parserOptions: {
        parser: '@typescript-eslint/parser',
        jsxPragma: 'React',
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    rules: {
      '@typescript-eslint/ban-ts-ignore': 'off',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-var-requires': 'off',
      '@typescript-eslint/no-empty-function': 'off',
      '@typescript-eslint/no-unused-expressions': 'off',
      '@typescript-eslint/no-unsafe-function-type': 'off',
      '@typescript-eslint/no-duplicate-enum-values': 'off',
      '@typescript-eslint/no-unnecessary-type-constraint': 'off',
      '@typescript-eslint/no-wrapper-object-types': 'off',
      'vue/custom-event-name-casing': 'off',
      'vue/valid-template-root': 'off',
      'vue/no-reserved-component-names': 'off',
      'vue/require-v-for-key': 'off',
      'vue/valid-v-for': 'off',
      'vue/prefer-import-from-vue': 'off',
      'vue/html-self-closing': 'off',
      'no-use-before-define': 'off',
      'no-var': 'off',
      'prefer-const': 'off',
      '@typescript-eslint/no-use-before-define': 'off',
      '@typescript-eslint/ban-ts-comment': 'off',
      '@typescript-eslint/ban-types': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          varsIgnorePattern: '.*',
          args: 'none',
          caughtErrors: 'none',
        },
      ],
      'no-unused-vars': [
        'error',
        {
          varsIgnorePattern: '.*',
          args: 'none',
          caughtErrors: 'none',
        },
      ],
      'space-before-function-paren': 'off',
      'vue/multi-word-component-names': 'off',
      'vue/attributes-order': 'off',
      'vue/one-component-per-file': 'off',
      'vue/html-closing-bracket-newline': 'off',
      'vue/max-attributes-per-line': 'off',
      'vue/multiline-html-element-content-newline': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/attribute-hyphenation': 'off',
      'vue/require-default-prop': 'off',
      'vue/html-self-closing': 'off',
      '@typescript-eslint/no-this-alias': [
        'error',
        {
          allowDestructuring: false,
          allowedNames: ['that'],
        },
      ],
    },
  },
];
