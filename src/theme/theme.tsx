import {extendTheme} from '@chakra-ui/react'

const config = {
    initialColorMode: 'dark',
    useSystemColorMode: false,
}

const styles = {
    global: {
        '*::-webkit-scrollbar': {
            width: '0px',
            background: 'transparent',
            display: 'none',
        },
        html: {
            scrollBehavior: 'smooth',
            transition: 'all 500ms ease',
        },
        body: {
            backgroundColor: '#4b4949',
        },
    },
}


const theme = extendTheme({config, styles})

export default theme