import { useEffect, useState } from "react";
import { Box, Button, Card, CardContent, Stack, Typography } from "@mui/material";
import { invoke } from "@tauri-apps/api/core";

/**
 * Home page — invokes the Rust `run_sidecar` command which spawns the
 * shell sidecar binary, captures its stdout, and returns it.
 */
export default function HomePage() {
  const [version, setVersion] = useState<string>("");
  const [output, setOutput] = useState<string>("");

  useEffect(() => {
    invoke<string>("get_app_version")
      .then(setVersion)
      .catch(() => setVersion("(running outside Tauri)"));
  }, []);

  const handleRun = async () => {
    try {
      const result = await invoke<string>("run_sidecar", { args: ["--greet", "world"] });
      setOutput(result);
    } catch (e) {
      setOutput(`Error: ${e}`);
    }
  };

  return (
    <Stack spacing={3}>
      <Typography variant="h4">Home</Typography>
      <Card>
        <CardContent>
          <Typography variant="subtitle2" color="text.secondary">
            App version
          </Typography>
          <Typography variant="h6">{version || "loading..."}</Typography>
        </CardContent>
      </Card>
      <Box>
        <Button variant="contained" onClick={handleRun}>
          Run sidecar
        </Button>
        {output && (
          <Typography sx={{ mt: 2, fontFamily: "monospace", whiteSpace: "pre-wrap" }} variant="body2">
            {output}
          </Typography>
        )}
      </Box>
    </Stack>
  );
}
