import React, {useEffect, useState} from 'react';
import {
    Box, Checkbox,
    Container,
    Heading,
    Slider,
    SliderFilledTrack, SliderMark,
    SliderThumb,
    SliderTrack,
    VStack,
} from '@chakra-ui/react';
import {invoke} from '@tauri-apps/api/tauri';


function App() {

    const [active, setActive] = useState(false)
    const [ enabled, setEnabled ] = useState(false);
    const [threshold, setThreshold] = useState(30);
    const [input, setInput] = useState(0);

    async function listen() {
        const stream = await navigator.mediaDevices.getUserMedia({
            audio: true,
            video: false,
        });
        const audioContext = new AudioContext();
        const analyser = audioContext.createAnalyser();
        const microphone = audioContext.createMediaStreamSource(stream);
        const scriptProcessor = audioContext.createScriptProcessor(2048, 1, 1);

        analyser.smoothingTimeConstant = 0.8;
        analyser.fftSize = 32;

        microphone.connect(analyser);
        analyser.connect(scriptProcessor);
        scriptProcessor.connect(audioContext.destination);
        scriptProcessor.onaudioprocess = function () {
            const array = new Uint8Array(analyser.frequencyBinCount);
            analyser.getByteFrequencyData(array);
            const arraySum = array.reduce((a, value) => a + value, 0);
            const average = Math.round(arraySum / array.length);
            setInput(average);
        };
    }

    useEffect(() => {
        listen();
    }, []);

    useEffect(() => {
        (async() => {
            if(input >= threshold) {
                setActive(true);
                if(enabled) await invoke('mouse_click');
            } else {
                if(active) setActive(false);
            }
        })();
    }, [input]);

    return (
        <Container>
            <VStack>
                <Heading color={'white'} size={'3xl'}>VTC 4</Heading>
                <Heading color={'white'} size={'xs'}>This program works off your system default audio input device</Heading>
                <Heading size={'sm'} textAlign={'center'} color={'white'} > Created by Lizard and DrFineSir</Heading>
            </VStack>
            <VStack mt={8} spacing={10}>
                <Heading size={'md'} color={'white'}>Threshold Sensitivity</Heading>
                <Slider
                    onChangeEnd={(value) => setThreshold(value)} aria-label='slider-ex-1' defaultValue={30}>
                    <SliderMark color='white' value={0} mt='1' ml='-2.5' fontSize='sm'>
                        0
                    </SliderMark>
                    <SliderMark color='white' value={25} mt='1' ml='-2.5' fontSize='sm'>
                        25
                    </SliderMark>
                    <SliderMark color='white' value={50} mt='1' ml='-2.5' fontSize='sm'>
                        50
                    </SliderMark>
                    <SliderMark color='white' value={75} mt='1' ml='-2.5' fontSize='sm'>
                        75
                    </SliderMark>
                    <SliderMark color='white' value={100} mt='1' ml='-2.5' fontSize='sm'>
                        100
                    </SliderMark>
                    <SliderMark
                        value={threshold}
                        textAlign='center'
                        bg='blue.500'
                        rounded='full'
                        color='white'
                        mt='-10'
                        ml='-6'
                        w='12'
                    >{threshold}
                    </SliderMark>
                    <SliderTrack >
                        <SliderFilledTrack />
                    </SliderTrack>
                    <SliderThumb />
                </Slider>
                <Box  textAlign='center' rounded='full' bg={active ? 'green.600' : 'red.600'} w={"100%"}><b>{active ? 'Would Click' : "Wouldndt Click"}</b></Box>
                <Checkbox onChange={() => setEnabled(!enabled)} color='white'><b>Enable the Clicky!</b></Checkbox>
            </VStack>
        </Container>
    );
}

export default App;
