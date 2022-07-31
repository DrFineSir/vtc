import React, {useEffect, useState} from 'react';
import {
    Box, Checkbox,
    Container,
    Heading, Progress, RangeSliderTrack,
    Slider,
    SliderFilledTrack, SliderMark,
    SliderThumb,
    SliderTrack,
    VStack,
} from '@chakra-ui/react';
import {invoke} from '@tauri-apps/api/tauri';
import {listen} from '@tauri-apps/api/event';


interface Payload {
    volume: number;
    met: boolean;
}


function App() {

    const [volumeD, setVolumeD] = useState(0);

    const [active, setActive] = useState(false)
    const [enable, setEnabled] = useState(false);
    const [threshold, setThreshold] = useState(30);

    // Sketchy work around to send the command one time once set
    const [once, setOnce] = useState(30);


    // ***************** useEffect *****************
    useEffect(() => {
        // On threshold change invoke command to set the threshold on the backend
        invoke('set_threshold', {threshold});
    }, [once]);

    useEffect(() => {
        (async () => {
            await listen<Payload>('threshold', ( event ) => {
                const { volume, met } = event.payload;
                setActive(met);
                setVolumeD(volume);
            });
        })();
    })

    useEffect(() => {
        // On enabled change invoke the command to set the enabled state on the backend
        invoke('set_enabled', {enable});
    }, [enable]);
    // ***************** useEffect *****************

    return (
        <Container>
            <VStack>
                <Heading color={'white'} size={'3xl'}>VTC 4</Heading>
                <Heading color={'white'} size={'xs'}>This program works off your system default audio input device</Heading>
                <Heading size={'sm'} textAlign={'center'} color={'white'} > Created by Lizard and DrFineSir</Heading>
            </VStack>
            <VStack mt={8} spacing={8}>
                <Heading size={'md'} color={'white'}>Threshold Sensitivity</Heading>
                <Slider
                    onChange={(value) => setThreshold(value)}
                    onChangeEnd={(value) => setOnce(value)}
                    aria-label='slider-ex-1' defaultValue={30}>
                    <SliderMark color='white' value={0} mt='2' ml='-2.5' fontSize='sm'>
                        0
                    </SliderMark>
                    <SliderMark color='white' value={25} mt='2' ml='-2.5' fontSize='sm'>
                        25
                    </SliderMark>
                    <SliderMark color='white' value={50} mt='2' ml='-2.5' fontSize='sm'>
                        50
                    </SliderMark>
                    <SliderMark color='white' value={75} mt='2' ml='-2.5' fontSize='sm'>
                        75
                    </SliderMark>
                    <SliderMark color='white' value={100} mt='2' ml='-2.5' fontSize='sm'>
                        100
                    </SliderMark>
                    <SliderMark
                        value={threshold}
                        textAlign='center'
                        bg='blue.500'
                        rounded='full'
                        color='white'
                        mt='-3'
                        ml='6'
                        w='12'
                    >{threshold}
                    </SliderMark>
                    <SliderTrack defaultValue={volumeD} >
                        <SliderFilledTrack />
                    </SliderTrack>
                    <SliderThumb />
                </Slider>
                <Progress width={'full'} value={volumeD} />
                <Box  textAlign='center' rounded='full' bg={active ? 'green.600' : 'red.600'} w={"100%"}><b>{active ? 'Would Click' : "Would not Click"}</b></Box>
                <Checkbox onChange={(e) => setEnabled(e.currentTarget.checked)} color='white'><b>Enable the Clicky!</b></Checkbox>
            </VStack>
        </Container>
    );
}

export default App;
