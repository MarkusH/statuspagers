// https://github.com/tailwindlabs/tailwindcss/blob/v2.2.19/stubs/defaultConfig.stub.js

const colors = require('tailwindcss/colors')

module.exports = {
  purge: {
    enabled: true,
    content: [
      "./templates/**/*.html"
    ],
  },
  darkMode: 'media',
  theme: {
    borderRadius: {
      none: '0',
      DEFAULT: '0.5rem',
      full: '9999px',
    },
    borderWidth: {
      '0': '0',
      '2': '2px',
    },
    colors: {
      current: colors.current,

      black: colors.black,
      white: colors.white,
      gray: {
        200: colors.gray[200],
        500: colors.gray[500],
      },
      green: colors.green[500],
      yellow: colors.yellow[500],
      red: colors.red[500],
    },
    fontSize: {
      lg: ['1.125rem', { lineHeight: '1.75rem' }],
      '4xl': ['2.25rem', { lineHeight: '2.5rem' }],
    },
    fontWeight: {
      bold: '700',
    },
    height: (theme) => ({
      ...theme('spacing')
    }),
    maxWidth: (theme, { breakpoints }) => ({
      ...breakpoints(theme('screens')),
    }),
    screens: {
      sm: '640px',
      md: '768px',
      lg: '1024px',
    },
    spacing: {
      '2': '0.5rem',
      '4': '1rem',
      '8': '2rem',
      '10': '2.5rem',
      '20': '5rem',
    },
    animation: false,
    blur: false,
    boxShadow: false,
    brightness: false,
    contrast: false,
    dropShadow: false,
    gradientColorStops: false,
    grayscale: false,
    gridAutoColumns: false,
    gridAutoFlow: false,
    gridAutoRows: false,
    gridColumn: false,
    gridColumnEnd: false,
    gridColumnStart: false,
    gridRow: false,
    gridRowEnd: false,
    gridRowStart: false,
    gridTemplateColumns: false,
    gridTemplateRows: false,
    hueRotate: false,
    inset: false,
    invert: false,
    letterSpacing: false,
    lineHeight: false,
    objectPosition: false,
    opacity: false,
    order: false,
    placeholderColor: false,
    ringColor: false,
    ringOffsetColor: false,
    ringOffsetWidth: false,
    ringWidth: false,
    rotate: false,
    saturate: false,
    scale: false,
    sepia: false,
    skew: false,
    space: false,
    transformOrigin: false,
    transitionDelay: false,
    transitionDuration: false,
    transitionProperty: false,
    transitionTimingFunction: false,
    translate: false,
    width: false,
    zIndex: false,


  },
  variants: {
    extend: {},
    backgroundColor: ['responsive', 'dark'],
    textColor: ['responsive', 'dark'],
  },
  corePlugins: {
    backgroundBlendMode: false,
    mixBlendMode: false,
  },
  plugins: [],
}
