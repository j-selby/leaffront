pipeline {
  agent any
  stages {
    stage('Build') {
      steps {
        sh 'cd station && cargo build --release --features glutin'
      }
    }
    stage('') {
      steps {
        archiveArtifacts 'target/release/leaffront-station'
      }
    }
  }
}