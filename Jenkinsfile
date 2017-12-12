pipeline {
  agent any
  stages {
    stage('Build') {
      steps {
        sh 'cd station && cargo build --release --features glutin'
      }
    }
    stage('Archive') {
      steps {
        archiveArtifacts 'station/target/release/leaffront-station'
      }
    }
  }
}